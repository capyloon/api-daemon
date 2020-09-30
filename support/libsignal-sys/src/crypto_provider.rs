
use crate::generated::ffi::*;
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};
use crypto::mac::Mac;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::sha2::Sha256;
use crypto::blockmodes::PkcsPadding;
use crypto::aes::{cbc_decryptor, cbc_encryptor, ctr, KeySize};
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use std::slice;
use std::mem;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;

// cipher used for AES encryption and decryption.
const SG_CIPHER_AES_CTR_NOPADDING: c_int = 1;
const SG_CIPHER_AES_CBC_PKCS5: c_int = 2;

/// Callback for a secure random number generator.
/// This function shall fill the provided buffer with random bytes.
///
/// @param data pointer to the output buffer
/// @param len size of the output buffer
/// @return 0 on success, negative on failure
extern "C" fn random_func(data: *mut u8, len: usize, _user_data: *mut c_void) -> c_int {
    // debug!("random_func len={}", len);
    let rng = SystemRandom::new();

    let array = unsafe { slice::from_raw_parts_mut(data, len) };

    match rng.fill(array) {
        Ok(_) => 0,
        Err(_) => -1, // TODO: check return error types.
    }
}

struct HmacWrapper<D: Digest> {
    ctxt: Hmac<D>,
}

impl<D: Digest> HmacWrapper<D> {
    fn new(digest: D, key: &[u8]) -> Self {
        HmacWrapper {
            ctxt: Hmac::new(digest, key),
        }
    }
}

/// Callback for an HMAC-SHA256 implementation.
/// This function shall initialize an HMAC context with the provided key.
///
/// @param hmac_context private HMAC context pointer
/// @param key pointer to the key
/// @param key_len length of the key
/// @return 0 on success, negative on failure
extern "C" fn hmac_sha256_init_func(
    hmac_context: *mut *mut c_void,
    key: *const u8,
    key_len: usize,
    _user_data: *mut c_void,
) -> c_int {
    debug!("hmac_sha256_init_func");
    let hkey = unsafe { slice::from_raw_parts_mut(key as *mut u8, key_len) };
    let hmac = HmacWrapper::new(Sha256::new(), hkey);

    // "leak" the pointer to let the C side manage its lifetime.
    let addr = Box::into_raw(Box::new(hmac)) as *mut c_void;
    // copy the address of our pointer to the right place.
    unsafe { *hmac_context = addr };
    0
}

/// Callback for an HMAC-SHA256 implementation.
/// This function shall update the HMAC context with the provided data
///
/// @param hmac_context private HMAC context pointer
/// @param data pointer to the data
/// @param data_len length of the data
/// @return 0 on success, negative on failure
extern "C" fn hmac_sha256_update_func(
    hmac_context: *mut c_void,
    data: *const u8,
    data_len: usize,
    _user_data: *mut c_void,
) -> c_int {
    debug!("hmac_sha256_update_func");
    let wrapper: &mut HmacWrapper<Sha256> = unsafe { mem::transmute(hmac_context as usize) };
    let array = unsafe { slice::from_raw_parts_mut(data as *mut _, data_len) };
    wrapper.ctxt.input(array);
    0
}

/// Callback for an HMAC-SHA256 implementation.
/// This function shall finalize an HMAC calculation and populate the output
/// buffer with the result.
///
/// @param hmac_context private HMAC context pointer
/// @param output buffer to be allocated and populated with the result
/// @return 0 on success, negative on failure
extern "C" fn hmac_sha256_final_func(
    hmac_context: *mut c_void,
    output: *mut *mut signal_buffer,
    _user_data: *mut c_void,
) -> c_int {
    debug!("hmac_sha256_final_func");
    let wrapper: &mut HmacWrapper<Sha256> = unsafe { mem::transmute(hmac_context as usize) };
    let mut buffer: [u8; 32] = [0; 32];
    wrapper.ctxt.raw_result(&mut buffer);

    unsafe {
        *output = signal_buffer::from_slice(&buffer);
    }
    0
}

/// Callback for an HMAC-SHA256 implementation.
/// This function shall free the private context allocated in
/// hmac_sha256_init_func.
///
/// @param hmac_context private HMAC context pointer
extern "C" fn hmac_sha256_cleanup_func(hmac_context: *mut c_void, _user_data: *mut c_void) {
    debug!("hmac_sha256_cleanup_func");
    // This transfers ownership of the service back to Rust which will drop it.
    let _wrapper: Box<HmacWrapper<Sha256>> =
        unsafe { Box::from_raw(hmac_context as *mut HmacWrapper<Sha256>) };
}

struct DigestWrapper {
    pub ctxt: digest::Context,
}

impl DigestWrapper {
    fn new() -> Self {
        DigestWrapper {
            ctxt: digest::Context::new(&digest::SHA512),
        }
    }
}

/// Callback for a SHA512 message digest implementation.
/// This function shall initialize a digest context.
///
/// @param digest_context private digest context pointer
/// @return 0 on success, negative on failure
extern "C" fn sha512_digest_init_func(
    digest_context: *mut *mut c_void,
    _user_data: *mut c_void,
) -> c_int {
    debug!("sha512_digest_init_func");
    // "leak" the pointer to let the C side manage its lifetime.
    let addr = Box::into_raw(Box::new(DigestWrapper::new())) as *mut c_void;
    // copy the address of our pointer to the right place.
    unsafe { *digest_context = addr };

    0
}

/// Callback for a SHA512 message digest implementation.
/// This function shall update the digest context with the provided data.
///
/// @param digest_context private digest context pointer
/// @param data pointer to the data
/// @param data_len length of the data
/// @return 0 on success, negative on failure
extern "C" fn sha512_digest_update_func(
    digest_context: *mut c_void,
    data: *const u8,
    data_len: usize,
    _user_data: *mut c_void,
) -> c_int {
    debug!("sha512_digest_update_func");
    let wrapper: &mut DigestWrapper = unsafe { mem::transmute(digest_context as usize) };
    let array = unsafe { slice::from_raw_parts_mut(data as *mut _, data_len) };
    wrapper.ctxt.update(array);
    0
}

/// Callback for a SHA512 message digest implementation.
/// This function shall finalize the digest calculation, populate the
/// output buffer with the result, and prepare the context for reuse.
///
/// @param digest_context private digest context pointer
/// @param output buffer to be allocated and populated with the result
/// @return 0 on success, negative on failure
extern "C" fn sha512_digest_final_func(
    digest_context: *mut c_void,
    output: *mut *mut signal_buffer,
    _user_data: *mut c_void,
) -> c_int {
    debug!("sha512_digest_final_func");
    let wrapper: &DigestWrapper = unsafe { mem::transmute(digest_context as usize) };
    let a = wrapper.ctxt.clone();
    let digest = a.finish();
    let slice = digest.as_ref();

    unsafe {
        *output = signal_buffer::from_slice(&slice);
    }
    0
}

/// Callback for a SHA512 message digest implementation.
/// This function shall free the private context allocated in
/// sha512_digest_init_func.
///
/// @param digest_context private digest context pointer
extern "C" fn sha512_digest_cleanup_func(digest_context: *mut c_void, _user_data: *mut c_void) {
    debug!("sha512_digest_final_func");
    // This transfers ownership of the service back to Rust which will drop it.
    let _wrapper: Box<DigestWrapper> =
        unsafe { Box::from_raw(digest_context as *mut DigestWrapper) };
}

/// Callback for an AES encryption implementation.
///
/// @param output buffer to be allocated and populated with the ciphertext
/// @param cipher specific cipher variant to use, either SG_CIPHER_AES_CTR_NOPADDING
/// or SG_CIPHER_AES_CBC_PKCS5
/// @param key the encryption key
/// @param key_len length of the encryption key
/// @param iv the initialization vector
/// @param iv_len length of the initialization vector
/// @param plaintext the plaintext to encrypt
/// @param plaintext_len length of the plaintext
/// @return 0 on success, negative on failure
extern "C" fn encrypt_func(
    output: *mut *mut signal_buffer,
    cipher: c_int,
    key: *const u8,
    key_len: usize,
    iv: *const u8,
    iv_len: usize,
    plaintext: *const u8,
    plaintext_len: usize,
    _user_data: *mut c_void,
) -> c_int {
    debug!(
        "encrypt_func key_len is {}, plaintext size is {}",
        key_len,
        plaintext_len
    );
    let ekey = unsafe { slice::from_raw_parts_mut(key as *mut u8, key_len) };
    let eiv = unsafe { slice::from_raw_parts_mut(iv as *mut u8, iv_len) };
    let eplain = unsafe { slice::from_raw_parts_mut(plaintext as *mut u8, plaintext_len) };

    if cipher == SG_CIPHER_AES_CBC_PKCS5 {
        let mut encryptor = cbc_encryptor(KeySize::KeySize256, ekey, eiv, PkcsPadding);
        let mut buffer = [0; 4096];
        let mut final_result = Vec::<u8>::new();
        let mut write_buffer = RefWriteBuffer::new(&mut buffer);
        let mut read_buffer = RefReadBuffer::new(eplain);
        loop {
            if let Ok(result) = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true) {
                // "write_buffer.take_read_buffer().take_remaining()" means:
                // from the writable buffer, create a new readable buffer which
                // contains all data that has been written, and then access all
                // of that data as a slice.
                final_result.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                        .iter()
                        .cloned(),
                );

                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            } else {
                return -3; // TODO: figure out a better error code.
            }
        }
        // If we haven't returned early from Error, set the output to
        // the content of the buffer.
        // Create a signal buffer and copy the Rust slice into it.
        unsafe {
            *output = signal_buffer::from_slice(&final_result);
        }
    } else if cipher == SG_CIPHER_AES_CTR_NOPADDING {
        let mut encryptor = ctr(KeySize::KeySize256, ekey, eiv);
        let mut buffer = vec![];
        encryptor.process(eplain, &mut buffer);
        unsafe {
            *output = signal_buffer::from_slice(&buffer);
        }
    } else {
        error!("Unexpected cipher: {}", cipher);
        // TODO: check error return codes.
        return -1;
    }

    0
}

/// Callback for an AES decryption implementation.
///
/// @param output buffer to be allocated and populated with the plaintext
/// @param cipher specific cipher variant to use, either SG_CIPHER_AES_CTR_NOPADDING
/// or SG_CIPHER_AES_CBC_PKCS5
/// @param key the encryption key
/// @param key_len length of the encryption key
/// @param iv the initialization vector
/// @param iv_len length of the initialization vector
/// @param ciphertext the ciphertext to decrypt
/// @param ciphertext_len length of the ciphertext
/// @return 0 on success, negative on failure
extern "C" fn decrypt_func(
    output: *mut *mut signal_buffer,
    cipher: c_int,
    key: *const u8,
    key_len: usize,
    iv: *const u8,
    iv_len: usize,
    ciphertext: *const u8,
    ciphertext_len: usize,
    _user_data: *mut c_void,
) -> c_int {
    debug!("decrypt_func key_len is {}", key_len);
    let ekey = unsafe { slice::from_raw_parts_mut(key as *mut u8, key_len) };
    let eiv = unsafe { slice::from_raw_parts_mut(iv as *mut u8, iv_len) };
    let ecipher = unsafe { slice::from_raw_parts_mut(ciphertext as *mut u8, ciphertext_len) };

    if cipher == SG_CIPHER_AES_CBC_PKCS5 {
        let mut decryptor = cbc_decryptor(KeySize::KeySize256, ekey, eiv, PkcsPadding);
        let mut buffer = [0; 4096];
        let mut final_result = Vec::<u8>::new();
        let mut write_buffer = RefWriteBuffer::new(&mut buffer);
        let mut read_buffer = RefReadBuffer::new(ecipher);
        loop {
            if let Ok(result) = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true) {
                // "write_buffer.take_read_buffer().take_remaining()" means:
                // from the writable buffer, create a new readable buffer which
                // contains all data that has been written, and then access all
                // of that data as a slice.
                final_result.extend(
                    write_buffer
                        .take_read_buffer()
                        .take_remaining()
                );

                match result {
                    BufferResult::BufferUnderflow => break,
                    BufferResult::BufferOverflow => {}
                }
            } else {
                return -3; // TODO: figure out a better error code.
            }
        }

        // If we haven't returned early from Error, set the output to
        // the content of the buffer.
        // Create a signal buffer and copy the Rust slice into it.
        unsafe {
            *output = signal_buffer::from_slice(&final_result);
        }
    } else if cipher == SG_CIPHER_AES_CTR_NOPADDING {
        let mut decryptor = ctr(KeySize::KeySize256, ekey, eiv);
        let mut buffer = vec![];
        decryptor.process(ecipher, &mut buffer);
        unsafe {
            *output = signal_buffer::from_slice(&buffer);
        }
    } else {
        error!("Unexpected cipher: {}", cipher);
        // TODO: check error return codes.
        return -1;
    }
    0
}

pub fn get_crypto_provider() -> signal_crypto_provider {
    signal_crypto_provider {
        random_func: Some(random_func),
        hmac_sha256_init_func: Some(hmac_sha256_init_func),
        hmac_sha256_update_func: Some(hmac_sha256_update_func),
        hmac_sha256_final_func: Some(hmac_sha256_final_func),
        hmac_sha256_cleanup_func: Some(hmac_sha256_cleanup_func),
        sha512_digest_init_func: Some(sha512_digest_init_func),
        sha512_digest_update_func: Some(sha512_digest_update_func),
        sha512_digest_final_func: Some(sha512_digest_final_func),
        sha512_digest_cleanup_func: Some(sha512_digest_cleanup_func),
        encrypt_func: Some(encrypt_func),
        decrypt_func: Some(decrypt_func),
        user_data: null_mut(),
    }
}
