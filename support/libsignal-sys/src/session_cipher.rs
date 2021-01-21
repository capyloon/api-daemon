use crate::generated::ffi::*;
use crate::signal_context::SignalContext;
use crate::store_context::StoreContext;
use std::cell::Cell;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;

// Wrapper around a session_cipher
pub type SessionCipherPtr = *mut session_cipher;
pub type CiphertextMessagePtr = *mut ciphertext_message;

// Returns a negative value in case of failure.
pub type DecyryptCallback = unsafe extern "C" fn(
    cipher: *mut session_cipher,
    plaintext: *mut signal_buffer,
    decrypt_context: *mut c_void,
) -> c_int;

pub struct SessionCipher {
    native: SessionCipherPtr,
    decrypt_context: Cell<*mut c_void>,
}

unsafe impl Sync for SessionCipher {}
unsafe impl Send for SessionCipher {}

impl SessionCipher {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(
        store: &StoreContext,
        remote_address: *const signal_protocol_address,
        ctxt: &SignalContext,
    ) -> Option<Self> {
        let mut cipher: SessionCipherPtr = null_mut();
        unsafe {
            if session_cipher_create(&mut cipher, store.native(), remote_address, ctxt.native())
                == 0
            {
                return Some(SessionCipher {
                    native: cipher,
                    decrypt_context: Cell::new(null_mut()),
                });
            } else {
                debug!("Error in session_cipher_create");
            }
        }

        None
    }

    pub fn set_callback(&self, callback: DecyryptCallback, decrypt_context: *mut c_void) {
        unsafe {
            session_cipher_set_decryption_callback(self.native, Some(callback));
        }
        self.decrypt_context.set(decrypt_context);
    }

    pub fn native(&self) -> SessionCipherPtr {
        self.native
    }

    // Returns <(message_type, message), error>
    pub fn encrypt(&self, message: &[u8]) -> Result<(u32, Vec<u8>), c_int> {
        let mut encrypted: CiphertextMessagePtr = null_mut();
        unsafe {
            let res = session_cipher_encrypt(
                self.native,
                message.as_ptr(),
                message.len() as _,
                &mut encrypted,
            );
            if res == 0 {
                let vec = (*(*encrypted).serialized).data_slice().to_vec();
                let m_type = (*encrypted).message_type as u32;
                ciphertext_message_destroy(encrypted);
                Ok((m_type, vec))
            } else {
                error!("session_cipher_encrypt failed: {}", res);
                Err(res)
            }
        }
    }

    // Returns the plain text.
    pub fn decrypt_message(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DecryptionError> {
        let mut plaintext: SignalBufferPtr = null_mut();
        unsafe {
            if let Some(gctxt) = SignalContext::new() {
                // First, create a pre_key_signal_message from the serialized ciphertext.
                type SignalMessagePtr = *mut signal_message;
                let mut message: SignalMessagePtr = null_mut();
                if signal_message_deserialize(
                    &mut message,
                    ciphertext.as_ptr(),
                    ciphertext.len() as _,
                    gctxt.native(),
                ) == 0
                {
                    // Then decrypt it.
                    let res = session_cipher_decrypt_signal_message(
                        self.native,
                        message,
                        self.decrypt_context.get(),
                        &mut plaintext,
                    );
                    signal_message_destroy(message as *mut signal_type_base);
                    if res == 0 {
                        let ret = (*plaintext).data_slice().to_vec();
                        signal_buffer_free(plaintext as *mut signal_buffer);
                        Ok(ret)
                    } else {
                        Err(res.into())
                    }
                } else {
                    Err(DecryptionError::DeserializationError)
                }
            } else {
                Err(DecryptionError::Other)
            }
        }
    }

    // Returns the plain text.
    pub fn decrypt_pre_key_message(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DecryptionError> {
        let mut plaintext: SignalBufferPtr = null_mut();
        unsafe {
            if let Some(gctxt) = SignalContext::new() {
                // First, create a pre_key_signal_message from the serialized ciphertext.
                type PreKeySignalMessagePtr = *mut pre_key_signal_message;
                let mut message: PreKeySignalMessagePtr = null_mut();
                if pre_key_signal_message_deserialize(
                    &mut message,
                    ciphertext.as_ptr(),
                    ciphertext.len() as _,
                    gctxt.native(),
                ) == 0
                {
                    // Then decrypt it.
                    let res = session_cipher_decrypt_pre_key_signal_message(
                        self.native,
                        message,
                        self.decrypt_context.get(),
                        &mut plaintext,
                    );
                    pre_key_signal_message_destroy(message as *mut signal_type_base);
                    if res == 0 {
                        let ret = (*plaintext).data_slice().to_vec();
                        signal_buffer_free(plaintext as *mut signal_buffer);
                        Ok(ret)
                    } else {
                        Err(res.into())
                    }
                } else {
                    Err(DecryptionError::DeserializationError)
                }
            } else {
                Err(DecryptionError::Other)
            }
        }
    }

    // Returns the remote registration id.
    pub fn remote_registration_id(&self) -> Option<u32> {
        let mut res: u32 = 0;
        unsafe {
            if session_cipher_get_remote_registration_id(self.native, &mut res) < 0 {
                return None;
            }
        }
        Some(res)
    }
}

impl Drop for SessionCipher {
    fn drop(&mut self) {
        debug!("Dropping SessionCipher");
        unsafe {
            session_cipher_free(self.native);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::signal_context::SignalContext;
    use crate::store_context::StoreContext;

    extern "C" fn decrypt_callback(
        _cipher: *mut session_cipher,
        _plaintext: *mut signal_buffer,
        _decrypt_context: *mut c_void,
    ) -> c_int {
        0
    }

    #[test]
    fn session_cipher_creation() {
        // Create all contexts and cipher.
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let address = signal_protocol_address::new("+1234567890", 42);
        let session_cipher = SessionCipher::new(&s_context, &address, &g_context).unwrap();
        session_cipher.set_callback(decrypt_callback, null_mut());
        address.destroy();
    }
}
