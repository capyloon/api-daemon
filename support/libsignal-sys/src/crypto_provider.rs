use crate::generated::ffi::*;
use block_modes::block_padding::Pkcs7;
use block_modes::cipher::{NewCipher, StreamCipher};
use block_modes::{BlockMode, Cbc};
use hmac::{Hmac, Mac, NewMac};
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::Sha256;
use std::mem;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use std::slice;

type Aes256Cbc = Cbc<aes::Aes256, Pkcs7>;
type Aes128Ctr = ctr::Ctr128BE<aes::Aes128>;

// cipher used for AES encryption and decryption.
const SG_CIPHER_AES_CTR_NOPADDING: c_int = 1;
const SG_CIPHER_AES_CBC_PKCS5: c_int = 2;

/// Callback for a secure random number generator.
/// This function shall fill the provided buffer with random bytes.
///
/// @param data pointer to the output buffer
/// @param len size of the output buffer
/// @return 0 on success, negative on failure
extern "C" fn random_func(data: *mut u8, len: size_t, _user_data: *mut c_void) -> c_int {
    // debug!("random_func len={}", len);
    let rng = SystemRandom::new();

    let array = unsafe { slice::from_raw_parts_mut(data, len as _) };

    match rng.fill(array) {
        Ok(_) => 0,
        Err(_) => -1, // TODO: check return error types.
    }
}

type HmacSha256 = Hmac<Sha256>;

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
    key_len: size_t,
    _user_data: *mut c_void,
) -> c_int {
    debug!("hmac_sha256_init_func");
    let hkey = unsafe { slice::from_raw_parts_mut(key as *mut u8, key_len as _) };

    match HmacSha256::new_varkey(hkey) {
        Ok(hmac) => {
            // "leak" the pointer to let the C side manage its lifetime.
            let addr = Box::into_raw(Box::new(hmac)) as *mut c_void;
            // copy the address of our pointer to the right place.
            unsafe { *hmac_context = addr };
            0
        }
        Err(err) => {
            error!("Failure in HmacSha256::new_from_sliceskey() : {}", err);
            -1
        }
    }
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
    data_len: size_t,
    _user_data: *mut c_void,
) -> c_int {
    debug!("hmac_sha256_update_func");
    let wrapper: &mut HmacSha256 = unsafe { &mut *(hmac_context as *mut Hmac<Sha256>) };
    let array = unsafe { slice::from_raw_parts_mut(data as *mut _, data_len as _) };
    wrapper.update(array);
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
    let wrapper: &mut HmacSha256 = unsafe { &mut *(hmac_context as *mut Hmac<Sha256>) };
    let result = wrapper.finalize_reset();

    unsafe {
        *output = signal_buffer::from_slice(&result.into_bytes());
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
    let _wrapper: Box<HmacSha256> = unsafe { Box::from_raw(hmac_context as *mut HmacSha256) };
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
    data_len: size_t,
    _user_data: *mut c_void,
) -> c_int {
    debug!("sha512_digest_update_func");
    let wrapper: &mut DigestWrapper = unsafe { mem::transmute(digest_context as usize) };
    let array = unsafe { slice::from_raw_parts_mut(data as *mut _, data_len as _) };
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
    key_len: size_t,
    iv: *const u8,
    iv_len: size_t,
    plaintext: *const u8,
    plaintext_len: size_t,
    _user_data: *mut c_void,
) -> c_int {
    debug!(
        "encrypt_func key_len is {}, plaintext size is {}",
        key_len, plaintext_len
    );
    let ekey = unsafe { slice::from_raw_parts(key as *mut u8, key_len as _) };
    let eiv = unsafe { slice::from_raw_parts(iv as *mut u8, iv_len as _) };
    let eplain = unsafe { slice::from_raw_parts_mut(plaintext as *mut u8, plaintext_len as _) };

    if cipher == SG_CIPHER_AES_CBC_PKCS5 {
        let cipher = match Aes256Cbc::new_from_slices(&ekey, &eiv) {
            Ok(cipher) => cipher,
            Err(err) => {
                error!("Failure in Aes256Cbc::new_from_slices() : {}", err);
                return -3;
            }
        };

        let final_result = cipher.encrypt_vec(eplain);

        // If we haven't returned early from Error, set the output to
        // the content of the buffer.
        // Create a signal buffer and copy the Rust slice into it.
        unsafe {
            *output = signal_buffer::from_slice(&final_result);
        }
    } else if cipher == SG_CIPHER_AES_CTR_NOPADDING {
        let mut cipher = Aes128Ctr::new(ekey.into(), eiv.into());
        cipher.apply_keystream(eplain);

        unsafe {
            *output = signal_buffer::from_slice(&eplain);
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
    key_len: size_t,
    iv: *const u8,
    iv_len: size_t,
    ciphertext: *const u8,
    ciphertext_len: size_t,
    _user_data: *mut c_void,
) -> c_int {
    debug!("decrypt_func key_len is {}", key_len);
    let ekey = unsafe { slice::from_raw_parts(key as *mut u8, key_len as _) };
    let eiv = unsafe { slice::from_raw_parts(iv as *mut u8, iv_len as _) };
    let ecipher = unsafe { slice::from_raw_parts_mut(ciphertext as *mut u8, ciphertext_len as _) };

    if cipher == SG_CIPHER_AES_CBC_PKCS5 {
        let cipher = match Aes256Cbc::new_from_slices(&ekey, &eiv) {
            Ok(cipher) => cipher,
            Err(err) => {
                error!("Failure in Aes256Cbc::new_from_slices() : {}", err);
                return -3;
            }
        };

        let final_result = match cipher.decrypt_vec(ecipher) {
            Ok(final_result) => final_result,
            Err(err) => {
                error!("Failure in decrypt_vec() : {}", err);
                return -3;
            }
        };

        // If we haven't returned early from Error, set the output to
        // the content of the buffer.
        // Create a signal buffer and copy the Rust slice into it.
        unsafe {
            *output = signal_buffer::from_slice(&final_result);
        }
    } else if cipher == SG_CIPHER_AES_CTR_NOPADDING {
        let mut cipher = Aes128Ctr::new(ekey.into(), eiv.into());
        cipher.apply_keystream(ecipher);
        unsafe {
            *output = signal_buffer::from_slice(&ecipher);
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
