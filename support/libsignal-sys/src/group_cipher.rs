use crate::generated::ffi::*;
use crate::signal_context::{SignalContext, SignalContextPtr};
use crate::store_context::StoreContext;
use std::cell::Cell;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;

// Wrapper around a group cipher
pub type GroupCipherPtr = *mut group_cipher;
pub type CiphertextMessagePtr = *mut ciphertext_message;

// Returns a negative value in case of failure.
pub type GroupCipherDecyryptCallback = unsafe extern "C" fn(
    cipher: *mut group_cipher,
    plaintext: *mut signal_buffer,
    decrypt_context: *mut c_void,
) -> c_int;

pub struct GroupCipher {
    // Keep sender key for libsignal group cipher processing
    sender_key_name: Box<signal_protocol_sender_key_name>,
    native: GroupCipherPtr,
    native_ctxt: SignalContextPtr,
    decrypt_context: Cell<*mut c_void>,
}

unsafe impl Sync for GroupCipher {}
unsafe impl Send for GroupCipher {}

impl GroupCipher {
    pub fn new(
        store: &StoreContext,
        ctxt: &SignalContext,
        group_id: &str,
        sender_name: &str,
        device_id: i32,
    ) -> Option<Self> {
        let sender_key_name = Box::new(signal_protocol_sender_key_name::new(
            group_id,
            sender_name,
            device_id,
        ));

        let mut cipher: GroupCipherPtr = null_mut();
        let ptr = Box::into_raw(sender_key_name);
        // Libsignal will hold sender_key_name in group cipher session, don't copy them inside
        // We have to set it in self to keep its lifetime as long as group cipher.
        unsafe {
            let key_name: Box<signal_protocol_sender_key_name> = Box::from_raw(ptr);
            if group_cipher_create(&mut cipher, store.native(), ptr, ctxt.native()) == 0 {
                return Some(GroupCipher {
                    sender_key_name: key_name,
                    native: cipher,
                    native_ctxt: ctxt.native(),
                    decrypt_context: Cell::new(null_mut()),
                });
            } else {
                debug!("Error in group_cipher_create");
            }
        }

        None
    }

    pub fn set_callback(
        &self,
        callback: GroupCipherDecyryptCallback,
        decrypt_context: *mut c_void,
    ) {
        unsafe {
            group_cipher_set_decryption_callback(self.native, Some(callback));
        }
        self.decrypt_context.set(decrypt_context);
    }

    pub fn native(&self) -> GroupCipherPtr {
        self.native
    }

    // Returns <message, error>
    pub fn encrypt(&self, message: &[u8]) -> Result<Vec<u8>, c_int> {
        let mut encrypted: CiphertextMessagePtr = null_mut();
        unsafe {
            let res =
                group_cipher_encrypt(self.native, message.as_ptr(), message.len(), &mut encrypted);
            if res == 0 {
                let vec = (*(*encrypted).serialized).data_slice().to_vec().clone();
                ::std::mem::forget(encrypted);
                ciphertext_message_destroy(encrypted);
                Ok(vec)
            } else {
                error!("group_cipher_encrypt failed: {}", res);
                Err(res)
            }
        }
    }

    // Returns the plain text.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, DecryptionError> {
        let mut plaintext: SignalBufferPtr = null_mut();
        unsafe {
            // First, create a sender_key_message from the serialized ciphertext.
            type SenderKeyMessagePtr = *mut sender_key_message;
            let mut message: SenderKeyMessagePtr = null_mut();
            if sender_key_message_deserialize(
                &mut message,
                ciphertext.as_ptr(),
                ciphertext.len(),
                self.native_ctxt,
            ) == 0
            {
                // Then decrypt it.
                let res = group_cipher_decrypt(
                    self.native,
                    message,
                    self.decrypt_context.get(),
                    &mut plaintext,
                );
                sender_key_message_destroy(message as *mut signal_type_base);
                if res == 0 {
                    let ret = (*plaintext).data_slice().to_vec().clone();
                    signal_buffer_free(plaintext as *mut signal_buffer);
                    Ok(ret)
                } else {
                    Err(res.into())
                }
            } else {
                Err(DecryptionError::DeserializationError)
            }
        }
    }
}

impl Drop for GroupCipher {
    fn drop(&mut self) {
        debug!("Dropping GroupCipher");
        unsafe {
            group_cipher_free(self.native);
        }
        self.sender_key_name.destroy();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::signal_context::SignalContext;
    use crate::store_context::StoreContext;

    extern "C" fn decrypt_callback(
        _cipher: *mut group_cipher,
        _plaintext: *mut signal_buffer,
        _decrypt_context: *mut c_void,
    ) -> c_int {
        0
    }

    #[test]
    fn group_cipher_creation() {
        // Create all contexts and cipher.
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let cipher = GroupCipher::new(&s_context, &g_context, "group 1", "sender 1", 42).unwrap();
        cipher.set_callback(decrypt_callback, null_mut());
    }
}
