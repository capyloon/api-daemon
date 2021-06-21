// Downloads and decrypts content with minimal memory use by
// chunking the process.

use crate::generated::common::DownloadDecryptResult;
use aes::Aes256;
use cipher::generic_array::{ArrayLength, GenericArray};
use block_modes::block_padding::Padding;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use buf_redux::policy::MinBuffered;
use buf_redux::BufReader;
use core::slice;
use hmac::{Hmac, Mac, NewMac};
use log::error;
use ring::digest;
use sha2::Sha256;
use std::io::BufRead;

// Borrowed from https://github.com/RustCrypto/block-ciphers/blob/88902c713bface4d550dae971e602429a6cdc325/block-modes/src/utils.rs#L18
pub(crate) fn to_blocks<N>(data: &mut [u8]) -> &mut [GenericArray<u8, N>]
where
    N: ArrayLength<u8>,
{
    let n = N::to_usize();
    debug_assert!(data.len() % n == 0);

    #[allow(unsafe_code)]
    unsafe {
        slice::from_raw_parts_mut(data.as_ptr() as *mut GenericArray<u8, N>, data.len() / n)
    }
}

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub fn download_decrypt<C>(
    url: &str,
    iv: &[u8],
    cipher_key: &[u8],
    hmac_key: &[u8],
    num_ciphertext_bytes: i64,
    num_tail_bytes: i64,
    mut on_chunk_decrypted: C,
) -> Result<DownloadDecryptResult, String>
where
    C: FnMut(&[u8]),
{
    const AES_BLOCK_SIZE: usize = 16;
    const BUFFER_SIZE: usize = 64 * 1024; // 64k buffers.
    const CHUNK_SIZE: usize = 64 * 1024;

    // Various validation checks for the parameters.

    // Consider localhost as a secure origin (like web browsers do), which helps with
    // running tests.
    if !(url.starts_with("https://")
        || url.starts_with("http://localhost")
        || url.starts_with("http://127.0.0.1"))
    {
        return Err("bad_url".into());
    }
    if num_ciphertext_bytes <= 0 || (num_ciphertext_bytes % (AES_BLOCK_SIZE as i64) != 0) {
        return Err("bad_ciphertext_size".into());
    }
    if iv.len() != AES_BLOCK_SIZE {
        return Err("bad_iv".into());
    }
    if cipher_key.len() != 32 {
        return Err("bad_cipher_key".into());
    }
    if hmac_key.len() != 32 {
        return Err("bad_hmac_key".into());
    }
    if num_tail_bytes < 0 {
        return Err("bad_tail_size".into());
    }

    // Start the download. This is sync but this doesn't read the body yet.
    let response = match reqwest::blocking::get(url) {
        Err(err) => {
            // DNS errors are reported by Hyper as custom "Other" errors which is not
            // specific enough so we fallback on testing the string value :(
            let err_msg = format!("{}", err);
            if err_msg.contains("failed to lookup address information: Name or service not known") {
                return Err("dns_error".into());
            }

            // Timeout.
            if err.is_timeout() {
                return Err("connection_error".into());
            }

            // SSL errors
            if err_msg.contains("SSL routines") {
                return Err("tls_error".into());
            }

            // Failure to connect (eg. closed port)
            if err.is_request() {
                return Err("connection_error".into());
            }

            // Fallback to the string description.
            return Err(format!("{}", err));
        }
        Ok(r) => r,
    };
    // Check that we http status.
    if response.status() != 200 {
        return Err(format!("http_error={}", response.status().as_u16()));
    }
    // If content-length is set, check that this matches what we expect.
    if let Some(content_length) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
        let length: i64 = content_length.to_str().unwrap_or("0").parse().unwrap_or(0);
        if length != num_ciphertext_bytes + num_tail_bytes {
            return Err("bad_content_size".into());
        }
    }

    // Prepare the HMAC, initializing it with the IV.
    let mut hmac_ctxt = Hmac::<Sha256>::new_varkey(hmac_key).map_err(|_| "hmac_error")?;
    hmac_ctxt.update(&iv);

    // Prepare the SHA256 hasher.
    let mut sha256_hasher = digest::Context::new(&digest::SHA256);

    let mut tail = Vec::<u8>::new();
    let mut downloaded = 0;

    let num_ciphertext_bytes = num_ciphertext_bytes as usize;
    let mut reader =
        BufReader::with_capacity(BUFFER_SIZE, response).set_policy(MinBuffered(CHUNK_SIZE));

    // Prepare the AES cbc decryptor.
    let mut cipher = match Aes256Cbc::new_from_slices(&cipher_key, &iv) {
        Ok(cipher) => cipher,
        Err(err) => {
            error!("Failure in Aes256Cbc::new_var : {}", err);
            return Err("cipher_creation_error".into());
        }
    };

    loop {
        let consumed;
        match reader.fill_buf() {
            Ok(buffer) => {
                let size = if buffer.len() > CHUNK_SIZE {
                    CHUNK_SIZE
                } else {
                    buffer.len()
                };

                consumed = size;

                if size == 0 {
                    break;
                }

                let cipher_text_size = {
                    if downloaded + size <= num_ciphertext_bytes {
                        size
                    } else if downloaded >= num_ciphertext_bytes {
                        // Still reading past the ciphertext size.
                        let mut v = buffer[0..size].to_vec();
                        tail.append(&mut v);
                        0
                    } else {
                        // Reading some ciphertext and some tail.
                        let remainder = downloaded + size - num_ciphertext_bytes;
                        let mut v = buffer[size - remainder..size].to_vec();
                        tail.append(&mut v);
                        size - remainder
                    }
                };

                if cipher_text_size != 0 {
                    let cipher_text_part = &buffer[0..cipher_text_size];

                    let mut rw_buffer = cipher_text_part.to_vec();
                    cipher.decrypt_blocks(to_blocks(&mut rw_buffer));

                    // At EOF, manage padding.
                    if cipher_text_size != CHUNK_SIZE {
                        let n = Pkcs7::unpad(&rw_buffer)
                            .map_err(|_| "padding_error".to_owned())?
                            .len();
                        rw_buffer.truncate(n);
                    }

                    // Update the hash.
                    sha256_hasher.update(&rw_buffer);

                    // Update the HMAC.
                    hmac_ctxt.update(cipher_text_part);

                    // Send the decrypted part.
                    on_chunk_decrypted(&rw_buffer)
                }

                downloaded += size;
            }
            Err(err) => {
                // Failure to read from the http connection.
                error!("Download error for {}: {}", url, err);
                return Err("download_error".into());
            }
        }

        reader.consume(consumed);
    }

    Ok(DownloadDecryptResult {
        tail,
        plain_text_hash: sha256_hasher.finish().as_ref().to_vec(),
        hmac: hmac_ctxt.finalize_reset().into_bytes().to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::download_decrypt;
    use crate::generated::common::DownloadDecryptResult;
    use std::fs;
    use std::io::{Read, Write};
    use test_server::{HttpRequest, HttpResponse, TestServer};

    /// Reads file content into a binary result.
    pub fn read_file(file: &str) -> Result<Vec<u8>, ::std::io::Error> {
        let mut file = fs::File::open(file)?;
        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content);
        Ok(content)
    }

    // Setup a test environment, and starts a web server on a
    // different port to avoid collisions.
    fn setup_test() -> TestServer {
        let server = test_server::new("localhost:0", |req: HttpRequest| {
            match read_file(&format!("./test-fixtures/{}", req.path())) {
                Ok(content) => HttpResponse::Ok().body(content),
                Err(_) => HttpResponse::NotFound().body(format!("Not found: {}", req.path())),
            }
        });

        // Sending back the server so it's not dropped and shutdown.
        server.unwrap()
    }

    struct Params {
        url: String,
        iv: Vec<u8>,
        cipher_key: Vec<u8>,
        hmac_key: Vec<u8>,
        num_ciphertext_bytes: i64,
        num_tail_bytes: i64,
    }

    fn download_decrypt_params<C>(
        params: &Params,
        on_chunk_decrypted: C,
    ) -> Result<DownloadDecryptResult, String>
    where
        C: FnMut(&[u8]),
    {
        download_decrypt(
            &params.url,
            &params.iv,
            &params.cipher_key,
            &params.hmac_key,
            params.num_ciphertext_bytes,
            params.num_tail_bytes,
            on_chunk_decrypted,
        )
    }

    #[test]
    fn invalid_parameters() {
        let server = setup_test();
        let mut req = Params {
            url: "http://example:9000/data".into(),
            num_ciphertext_bytes: -10,
            iv: vec![],
            cipher_key: vec![],
            hmac_key: vec![],
            num_tail_bytes: -10,
        };

        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_url".to_owned());

        req.url = "https://example:9000/data".into();
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_ciphertext_size".to_owned());

        req.num_ciphertext_bytes = 77;
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_ciphertext_size".to_owned());

        req.num_ciphertext_bytes = 64;
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_iv".to_owned());

        req.iv = vec![0u8; 16];
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_cipher_key".to_owned());

        req.cipher_key = vec![0u8; 32];
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_hmac_key".to_owned());

        req.hmac_key = vec![0u8; 32];
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "bad_tail_size".to_owned());

        req.num_tail_bytes = 12;
        req.url = format!("{}/_not_here", server.url());
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "http_error=404");

        // Request to an invalid DNS name.
        // TODO: Re-enable once CI failures are understood.
        // url = "https://invalid.dns".into();
        // let res = download_decrypt(&req, |_| {}).err().unwrap();
        // assert_eq!(res, "dns_error");

        // Request to a closed tcp port.
        req.url = "http://localhost:9999/data".into();
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "connection_error");

        // https request on an http endpoint.
        req.url = "https://localhost:9000/data".into();
        let res = download_decrypt_params(&req, |_| {}).err().unwrap();
        assert_eq!(res, "connection_error");
    }

    #[test]
    fn example1() {
        let server = setup_test();
        let url = format!("{}/example1", server.url());

        let _ = fs::remove_file("./test-fixtures/result1");

        let req = Params {
            url,
            num_ciphertext_bytes: 9343520,
            iv: vec![
                0xBB, 0xCD, 0xCA, 0xD8, 0x42, 0x66, 0x86, 0xD3, 0x39, 0x38, 0x43, 0x91, 0x34, 0x2E,
                0x87, 0xA4,
            ],
            cipher_key: vec![
                0xE1, 0x94, 0xCC, 0xD3, 0x8E, 0x95, 0x09, 0x24, 0x7C, 0xCF, 0x87, 0xFB, 0x8D, 0xFA,
                0x58, 0xF4, 0x4B, 0xA2, 0xC8, 0xAC, 0x54, 0xAB, 0x26, 0x4C, 0xE2, 0x11, 0xD9, 0x81,
                0x6B, 0x0E, 0x96, 0xA2,
            ],
            hmac_key: vec![
                0x94, 0xAB, 0xC2, 0x36, 0x0C, 0x2C, 0x88, 0x82, 0xB2, 0xE5, 0x16, 0xDF, 0x14, 0x40,
                0xC7, 0x53, 0xC9, 0xB8, 0xE3, 0xC2, 0x3C, 0x5E, 0xFF, 0x16, 0x02, 0x5F, 0x75, 0xB7,
                0x4C, 0xD7, 0xDE, 0x7F,
            ],
            num_tail_bytes: 10,
        };

        let mut file = fs::File::create("./test-fixtures/result1").unwrap();

        let res = download_decrypt_params(&req, |buf| {
            let _ = file.write_all(buf);
        })
        .unwrap();

        let _ = file.flush();

        assert_eq!(res.tail.len(), 10);
        // In these particular examples, the 10 tail bytes should match the first 10 bytes of the generated hmac
        assert_eq!(res.tail, res.hmac[0..10].to_vec());
    }

    #[test]
    fn example2() {
        let server = setup_test();
        let url = format!("{}/example2", server.url());

        let req = Params {
            url,
            num_ciphertext_bytes: 82720,
            iv: vec![
                0xD1, 0x69, 0x86, 0x0C, 0x8A, 0xBD, 0x48, 0xD7, 0x87, 0xD8, 0xAE, 0xC9, 0x9C, 0x8C,
                0x29, 0x6A,
            ],
            cipher_key: vec![
                0x1C, 0xF0, 0x67, 0xFE, 0x60, 0xE7, 0xBE, 0xF4, 0x42, 0x01, 0x5B, 0xD2, 0x77, 0x48,
                0xBB, 0x4B, 0x10, 0xF3, 0xC3, 0x89, 0x3A, 0xF4, 0x51, 0xE7, 0xC2, 0x15, 0x17, 0xE2,
                0x48, 0x7D, 0xC2, 0xDD,
            ],
            hmac_key: vec![
                0xCF, 0xD5, 0x91, 0x68, 0x6D, 0x65, 0x25, 0x8A, 0x96, 0x37, 0x61, 0x58, 0x44, 0x52,
                0x98, 0x5E, 0xE0, 0x84, 0xE8, 0x1F, 0xE5, 0xA7, 0x28, 0x4D, 0x60, 0xF9, 0xDB, 0x5C,
                0xDC, 0x9D, 0x04, 0x9D,
            ],
            num_tail_bytes: 10,
        };

        let mut file = fs::File::create("./test-fixtures/result2").unwrap();

        let res = download_decrypt_params(&req, |buf| {
            let _ = file.write_all(buf);
        })
        .unwrap();

        let _ = file.flush();

        assert_eq!(res.tail.len(), 10);
        // In these particular examples, the 10 tail bytes should match the first 10 bytes of the generated hmac
        assert_eq!(res.tail, res.hmac[0..10].to_vec());
    }

    #[test]
    fn example3() {
        let server = setup_test();
        let url = format!("{}/example3", server.url());

        let req = Params {
            url,
            num_ciphertext_bytes: 3291648,
            iv: vec![
                23, 154, 27, 217, 23, 171, 155, 5, 191, 138, 216, 105, 212, 238, 136, 39,
            ],
            cipher_key: vec![
                217, 20, 56, 61, 17, 241, 40, 228, 180, 139, 47, 166, 237, 209, 148, 163, 143, 97,
                224, 164, 237, 181, 112, 109, 77, 248, 113, 132, 16, 157, 250, 10,
            ],
            hmac_key: vec![
                16, 67, 107, 57, 211, 105, 230, 158, 97, 178, 82, 164, 193, 64, 103, 76, 19, 131,
                132, 34, 224, 41, 222, 97, 7, 98, 229, 62, 230, 191, 178, 191,
            ],
            num_tail_bytes: 10,
        };

        let mut file = fs::File::create("./test-fixtures/result3").unwrap();

        let mut first = true;
        let res = download_decrypt_params(&req, |buf| {
            file.write_all(buf).unwrap();
            if first {
                // Check the beginning of the decrypted text.
                assert_eq!(b"The Project Gutenberg EBook", &buf[3..30]);
                first = false;
            }
        })
        .unwrap();

        file.flush().unwrap();
        assert_eq!(res.tail.len(), 10);

        // 3ed0f41cfdf660846878943bad5b9d575bcae1e4a92ee9a7f43d3c9dba2af344 <- sha256 sum
        let sha256sum = [
            0x3e, 0xd0, 0xf4, 0x1c, 0xfd, 0xf6, 0x60, 0x84, 0x68, 0x78, 0x94, 0x3b, 0xad, 0x5b,
            0x9d, 0x57, 0x5b, 0xca, 0xe1, 0xe4, 0xa9, 0x2e, 0xe9, 0xa7, 0xf4, 0x3d, 0x3c, 0x9d,
            0xba, 0x2a, 0xf3, 0x44,
        ];

        assert_eq!(res.plain_text_hash.len(), 32);
        assert_eq!(res.plain_text_hash, sha256sum);

        // In these particular examples, the 10 tail bytes should match the first 10 bytes of the generated hmac
        assert_eq!(res.tail, res.hmac[0..10].to_vec());
    }
}
