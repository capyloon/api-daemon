use crate::crypto_provider::get_crypto_provider;
use crate::generated::ffi::*;
use std::fmt;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null_mut;

// Wrapper around the Signal context.
pub type SignalContextPtr = *mut signal_context;

#[derive(Debug)]
pub struct SignalError;

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generic Signal Error")
    }
}

impl std::error::Error for SignalError {}

pub struct SignalContext {
    native: SignalContextPtr,
}

// Log levels defined in signal_protocol.h
// #define SG_LOG_ERROR   0
// #define SG_LOG_WARNING 1
// #define SG_LOG_NOTICE  2
// #define SG_LOG_INFO    3
// #define SG_LOG_DEBUG   4

extern "C" fn signal_log_rs(
    level: c_int,
    message: *const c_char,
    len: size_t,
    _user_data: *mut c_void,
) {
    let msg = unsafe { String::from_raw_parts(message as *mut u8, len as _, len as _) };

    match level {
        0 => error!("SignalLog: {}", msg),
        1 => warn!("SignalLog: {}", msg),
        2 | 3 => info!("SignalLog: {}", msg),
        4 => debug!("SignalLog: {}", msg),
        _ => debug!("SignalLog: {}", msg),
    }

    // We don't manage the string buffer.
    ::std::mem::forget(msg);
}

extern "C" fn signal_lock_rs(_user_data: *mut ::std::os::raw::c_void) {
    debug!("signal_lock_rs");
}

extern "C" fn signal_unlock_rs(_user_data: *mut ::std::os::raw::c_void) {
    debug!("signal_unlock_rs");
}

impl SignalContext {
    // Creates a new context, or return None if that failed.
    pub fn new() -> Option<Self> {
        let mut ctxt: SignalContextPtr = null_mut();
        if unsafe { signal_context_create(&mut ctxt, null_mut()) } == 0 {
            // Set up the log function.
            if unsafe { signal_context_set_log_function(ctxt, Some(signal_log_rs)) } != 0 {
                error!("Failed to set log function on context!");
            } else {
                debug!("Log function for context set up.");
            }

            // Set up the lock functions.
            if unsafe {
                signal_context_set_locking_functions(
                    ctxt,
                    Some(signal_lock_rs),
                    Some(signal_unlock_rs),
                )
            } != 0
            {
                error!("Failed to set locking functions on context!");
            } else {
                debug!("Locking functions for context set up.");
            }

            // Set up the crypto provider.
            let provider = get_crypto_provider();
            if unsafe { signal_context_set_crypto_provider(ctxt, &provider) } == 0 {
                return Some(SignalContext { native: ctxt });
            } else {
                debug!("Error in signal_context_set_crypto_provider");
            }
        } else {
            debug!("Error in signal_context_create");
        }

        None
    }

    pub fn native(&self) -> SignalContextPtr {
        self.native
    }

    // Returns the registration id.
    pub fn get_registration_id(&self, extended_range: bool) -> Result<u32, SignalError> {
        let mut id: u32 = 0;
        let ext: c_int = if extended_range { 1 } else { 0 };

        if unsafe { signal_protocol_key_helper_generate_registration_id(&mut id, ext, self.native) }
            == 0
        {
            return Ok(id);
        }
        Err(SignalError)
    }

    // Returns a (public_key, private_key)
    pub fn generate_identity_key_pair(&self) -> Result<(KeyArray, KeyArray), SignalError> {
        let mut key_pair: *mut ratchet_identity_key_pair = null_mut();
        unsafe {
            if signal_protocol_key_helper_generate_identity_key_pair(&mut key_pair, self.native)
                == 0
            {
                // Copy the keys and destroy the pair right away.
                let public_key = (*(*key_pair).public_key).data;
                let private_key = (*(*key_pair).private_key).data;
                ratchet_identity_key_pair_destroy(&mut (*key_pair).base);

                return Ok((public_key, private_key));
            }
        }

        Err(SignalError)
    }

    // Returns a Vec of (id, public_key, private_key)
    pub fn generate_pre_keys(
        &self,
        start: u32,
        count: u32,
    ) -> Result<Vec<(u32, KeyArray, KeyArray)>, SignalError> {
        let mut key_list: *mut signal_protocol_key_helper_pre_key_list_node = null_mut();
        unsafe {
            if signal_protocol_key_helper_generate_pre_keys(
                &mut key_list,
                start,
                count,
                self.native,
            ) == 0
            {
                // Iterate over the liste to build a vector of SessionPreKey
                let mut node = key_list;
                let mut res = vec![];
                while !node.is_null() {
                    let element = &*(*node).element;
                    let key_pair = *element.key_pair;
                    let item = (
                        element.id,
                        (*key_pair.public_key).data,
                        (*key_pair.private_key).data,
                    );
                    res.push(item);
                    node = (*node).next;
                }

                signal_protocol_key_helper_key_list_free(key_list);

                return Ok(res);
            }
        }

        Err(SignalError)
    }

    // Returns a (id, public_key, private_key, timestamp, signature)
    #[allow(clippy::field_reassign_with_default)]
    pub fn generate_signed_pre_key(
        &self,
        public_key: &[u8],
        private_key: &[u8],
        key_id: u32,
        timestamp: u64,
    ) -> Result<(u32, KeyArray, KeyArray, u64, Vec<u8>), SignalError> {
        let mut pre_key: *mut session_signed_pre_key = null_mut();
        build_key!(pub_key, ec_public_key, public_key);
        build_key!(priv_key, ec_private_key, private_key);

        struct KeyPairWrapper {
            key_pair: *mut ratchet_identity_key_pair,
        }

        impl Drop for KeyPairWrapper {
            fn drop(&mut self) {
                unsafe {
                    ratchet_identity_key_pair_destroy(&mut (*self.key_pair).base);
                }
            }
        }

        unsafe {
            let mut wrapper = KeyPairWrapper {
                key_pair: null_mut(),
            };

            ratchet_identity_key_pair_create(&mut wrapper.key_pair, &mut pub_key, &mut priv_key);

            if signal_protocol_key_helper_generate_signed_pre_key(
                &mut pre_key,
                wrapper.key_pair,
                key_id,
                timestamp,
                self.native,
            ) == 0
            {
                let pre_key_obj = &*pre_key;
                let key_pair = *pre_key_obj.key_pair;

                assert_eq!(pre_key_obj.signature_len, 64);
                let res = (
                    pre_key_obj.id,
                    (*key_pair.public_key).data,
                    (*key_pair.private_key).data,
                    pre_key_obj.timestamp,
                    pre_key_obj.signature.to_vec(),
                );

                session_pre_key_destroy(&mut (*pre_key).base);

                return Ok(res);
            }
        }

        Err(SignalError)
    }

    // Returns a (public_key, private_key)
    pub fn generate_sender_signing_key(&self) -> Result<(KeyArray, KeyArray), SignalError> {
        let mut key_pair: *mut ec_key_pair = null_mut();
        unsafe {
            if signal_protocol_key_helper_generate_sender_signing_key(&mut key_pair, self.native)
                == 0
            {
                let res = (
                    (*(*key_pair).public_key).data,
                    (*(*key_pair).private_key).data,
                );

                ec_key_pair_destroy(&mut (*key_pair).base);

                return Ok(res);
            }
        }

        Err(SignalError)
    }

    pub fn generate_sender_key(&self) -> Result<Vec<u8>, SignalError> {
        let mut buffer: *mut signal_buffer = null_mut();
        unsafe {
            if signal_protocol_key_helper_generate_sender_key(&mut buffer, self.native) == 0 {
                let res = (*buffer).data_slice().to_vec();

                signal_buffer_free(buffer);

                return Ok(res);
            }
        }
        Err(SignalError)
    }

    pub fn generate_sender_key_id(&self) -> Result<u32, SignalError> {
        unsafe {
            let mut res: u32 = 42;
            if signal_protocol_key_helper_generate_sender_key_id(&mut res, self.native) == 0 {
                return Ok(res);
            }
        }
        Err(SignalError)
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn curve_calculate_agreement(
        public_key: &[u8],
        private_key: &[u8],
    ) -> Result<KeyArray, SignalError> {
        // Check that the keys size is 32 bytes.
        if public_key.len() != 32 || private_key.len() != 32 {
            return Err(SignalError);
        }

        build_key!(pub_key, ec_public_key, public_key);
        build_key!(priv_key, ec_private_key, private_key);

        unsafe {
            type U8ptr = *mut u8;
            let mut res: U8ptr = null_mut();
            let out = curve_calculate_agreement(&mut res, &pub_key, &priv_key);
            // The return value is the length of the shared secret.
            if out == 32 {
                let slice = ::std::slice::from_raw_parts(res, 32);
                let mut array: KeyArray = [0; 32];
                array[..32].clone_from_slice(&slice[..32]);

                // We can use signal_buffer_free() to deallocate the returned array
                // because signal_buffer_free() is just a null-checked free().
                signal_buffer_free(res as *mut signal_buffer);

                Ok(array)
            } else {
                error!("curve_calculate_agreement failed with {}", out);
                Err(SignalError)
            }
        }
    }

    pub fn curve_verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let mut decoded_key: *mut ec_public_key = null_mut();
        let ctx = SignalContext::new().expect("Failed to create SignalContext !");

        // If the public key doesn't have the leading 0x05, add it first.
        let full_key = if public_key.len() == 33 {
            public_key.to_vec()
        } else {
            let mut tmp = vec![0x05];
            tmp.append(&mut public_key.to_vec());
            tmp
        };

        let decode_out = unsafe {
            curve_decode_point(
                &mut decoded_key,
                full_key.as_ptr(),
                full_key.len() as _,
                ctx.native(),
            )
        };
        if decode_out < 0 {
            error!("curve_decode_point failed {}", decode_out);
            return false;
        }

        let out = unsafe {
            curve_verify_signature(
                decoded_key,
                message.as_ptr(),
                message.len() as _,
                signature.as_ptr(),
                signature.len() as _,
            )
        };
        // Return 1 if valid, 0 if invalid, negative on failure
        if out < 0 {
            error!("curve_verify_signature failed with {}", out);
        }
        unsafe { (*decoded_key).unref() };

        out == 1
    }
}

impl Drop for SignalContext {
    fn drop(&mut self) {
        debug!("Dropping SignalContext");
        unsafe { signal_context_destroy(self.native) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_signal_context() {
        let context = SignalContext::new();
        assert!(context.is_some());
    }

    #[test]
    fn get_registration_id() {
        let context = SignalContext::new().unwrap();
        let id = context.get_registration_id(false);
        assert!(id.is_ok());
        assert_ne!(id.unwrap(), 0);
    }

    #[test]
    fn generate_identity_key_pair() {
        let context = SignalContext::new().unwrap();
        let (_public, _private) = context.generate_identity_key_pair().unwrap();
        // Not crashing is basically all we are testing here...
    }

    #[test]
    fn generate_pre_keys() {
        let context = SignalContext::new().unwrap();
        let res = context.generate_pre_keys(1, 4).unwrap();
        assert_eq!(res.len(), 4);
        let res = context.generate_pre_keys(10, 200).unwrap();
        assert_eq!(res.len(), 200);
    }

    #[test]
    fn generate_signed_pre_key() {
        let context = SignalContext::new().unwrap();
        let id = context.get_registration_id(false).unwrap();
        let (public, private) = context.generate_identity_key_pair().unwrap();
        let res = context
            .generate_signed_pre_key(&public, &private, id, 0)
            .unwrap();
        assert_eq!(res.0, id);
        assert_eq!(res.1.len(), 32);
        assert_eq!(res.2.len(), 32);
        assert_eq!(res.3, 0);
        assert_eq!(res.4.len(), 64);
    }

    #[test]
    fn generate_sender_signing_key() {
        let context = SignalContext::new().unwrap();
        context.generate_sender_signing_key().unwrap();
    }

    #[test]
    fn generate_sender_key() {
        let context = SignalContext::new().unwrap();
        let key = context.generate_sender_key().unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn generate_sender_key_id() {
        let context = SignalContext::new().unwrap();
        let id = context.generate_sender_key_id().unwrap();
        assert_ne!(id, 42);
    }

    #[test]
    fn curve_calculate_agreement() {
        let context = SignalContext::new().unwrap();
        let (public, private) = context.generate_identity_key_pair().unwrap();
        let _res = SignalContext::curve_calculate_agreement(&public, &private).unwrap();
    }

    #[test]
    fn curve_verify_signature() {
        // Create all contexts and builder.
        let alice_identity_public = [
            0x05, 0xab, 0x7e, 0x71, 0x7d, 0x4a, 0x16, 0x3b, 0x7d, 0x9a, 0x1d, 0x80, 0x71, 0xdf,
            0xe9, 0xdc, 0xf8, 0xcd, 0xcd, 0x1c, 0xea, 0x33, 0x39, 0xb6, 0x35, 0x6b, 0xe8, 0x4d,
            0x88, 0x7e, 0x32, 0x2c, 0x64,
        ];
        let alice_ephemeral_public = [
            0x05, 0xed, 0xce, 0x9d, 0x9c, 0x41, 0x5c, 0xa7, 0x8c, 0xb7, 0x25, 0x2e, 0x72, 0xc2,
            0xc4, 0xa5, 0x54, 0xd3, 0xeb, 0x29, 0x48, 0x5a, 0x0e, 0x1d, 0x50, 0x31, 0x18, 0xd1,
            0xa8, 0x2d, 0x99, 0xfb, 0x4a,
        ];
        let alice_signature = [
            0x5d, 0xe8, 0x8c, 0xa9, 0xa8, 0x9b, 0x4a, 0x11, 0x5d, 0xa7, 0x91, 0x09, 0xc6, 0x7c,
            0x9c, 0x74, 0x64, 0xa3, 0xe4, 0x18, 0x02, 0x74, 0xf1, 0xcb, 0x8c, 0x63, 0xc2, 0x98,
            0x4e, 0x28, 0x6d, 0xfb, 0xed, 0xe8, 0x2d, 0xeb, 0x9d, 0xcd, 0x9f, 0xae, 0x0b, 0xfb,
            0xb8, 0x21, 0x56, 0x9b, 0x3d, 0x90, 0x01, 0xbd, 0x81, 0x30, 0xcd, 0x11, 0xd4, 0x86,
            0xce, 0xf0, 0x47, 0xbd, 0x60, 0xb8, 0x6e, 0x88,
        ];

        let out = SignalContext::curve_verify_signature(
            &alice_identity_public,
            &alice_ephemeral_public,
            &alice_signature,
        );

        assert_eq!(out, true);

        let fake_alice_signature = [
            0x00, 0xe8, 0x8c, 0xa9, 0xa8, 0x9b, 0x4a, 0x11, 0x5d, 0xa7, 0x91, 0x09, 0xc6, 0x7c,
            0x9c, 0x74, 0x64, 0xa3, 0xe4, 0x18, 0x02, 0x74, 0xf1, 0xcb, 0x8c, 0x63, 0xc2, 0x98,
            0x4e, 0x28, 0x6d, 0xfb, 0xed, 0xe8, 0x2d, 0xeb, 0x9d, 0xcd, 0x9f, 0xae, 0x0b, 0xfb,
            0xb8, 0x21, 0x56, 0x9b, 0x3d, 0x90, 0x01, 0xbd, 0x81, 0x30, 0xcd, 0x11, 0xd4, 0x86,
            0xce, 0xf0, 0x47, 0xbd, 0x60, 0xb8, 0x6e, 0x88,
        ];
        let failout = SignalContext::curve_verify_signature(
            &alice_identity_public,
            &alice_ephemeral_public,
            &fake_alice_signature,
        );

        assert_eq!(failout, false);
    }

    #[test]
    fn test_wa_signature() {
        let wa_pub_key = [
            0xF7, 0x42, 0x48, 0xD2, 0xE8, 0xC4, 0x72, 0x06, 0x0F, 0x5B, 0x39, 0xE9, 0xE0, 0xCC,
            0x76, 0xC3, 0x38, 0x3F, 0xAB, 0x94, 0xF6, 0x91, 0x0F, 0xCC, 0xD0, 0xDB, 0x60, 0xB3,
            0x57, 0x5C, 0x69, 0x08,
        ];

        let wa_message = [
            0x08, 0x02, 0x12, 0x11, 0x57, 0x68, 0x61, 0x74, 0x73, 0x41, 0x70, 0x70, 0x4C, 0x6F,
            0x6E, 0x67, 0x54, 0x65, 0x72, 0x6D, 0x31, 0x22, 0x16, 0x43, 0x68, 0x61, 0x74, 0x20,
            0x53, 0x74, 0x61, 0x74, 0x69, 0x63, 0x20, 0x50, 0x75, 0x62, 0x6C, 0x69, 0x63, 0x20,
            0x4B, 0x65, 0x79, 0x2A, 0x20, 0xDA, 0xA5, 0x86, 0x97, 0xEF, 0x0B, 0x86, 0x8F, 0xA1,
            0x10, 0xA6, 0xDC, 0x1C, 0x33, 0xEC, 0xEE, 0x95, 0x85, 0x62, 0x6C, 0x9C, 0x3E, 0x75,
            0x04, 0xE8, 0x0C, 0x80, 0x13, 0xED, 0x59, 0xB2, 0x32,
        ];

        let wa_signature = [
            0x26, 0x91, 0x11, 0x82, 0xFB, 0x36, 0x80, 0x9B, 0x2A, 0xC3, 0xB9, 0x89, 0x84, 0x8A,
            0x1C, 0xA1, 0x1E, 0x9F, 0xC9, 0xBD, 0xDD, 0x6E, 0xAB, 0x71, 0xCA, 0x5C, 0x07, 0x26,
            0xCB, 0x1D, 0xFC, 0x0D, 0x99, 0x46, 0x62, 0x24, 0xEF, 0xA9, 0xA0, 0xD9, 0xC1, 0x72,
            0x55, 0x32, 0x04, 0x25, 0xE4, 0xE6, 0x66, 0xC0, 0x0F, 0x04, 0x3B, 0x00, 0x02, 0xAC,
            0xED, 0x1E, 0x98, 0xD8, 0x94, 0x2B, 0x05, 0x80,
        ];

        assert_eq!(
            SignalContext::curve_verify_signature(&wa_pub_key, &wa_message, &wa_signature),
            true
        );
    }
}
