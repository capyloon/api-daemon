use crate::generated::common::*;
use crate::store_context::StoreContext;
use common::traits::{SimpleObjectTracker, TrackerId};
use libsignal_sys::ffi::{group_cipher, signal_buffer, DecryptionError};
use libsignal_sys::{GroupCipher as FfiGroupCipher, SignalContext};
use log::debug;
use std::os::raw::{c_int, c_void};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

// Returns a negative value in case of failure.
pub struct GroupCipher {
    id: TrackerId,
    ffi: Arc<FfiGroupCipher>,
    #[allow(dead_code)] // We need to hold the callback proxy alive with the same lifetime.
    callback: Rc<DecryptionCallbackProxy>,
    #[allow(dead_code)] // We need to hold the store context alive with the same lifetime.
    store_context: StoreContext,
}

impl Drop for GroupCipher {
    fn drop(&mut self) {
        debug!("Dropping GroupCipher #{}", self.id);
    }
}

extern "C" fn native_decrypt_callback(
    _cipher: *mut group_cipher,
    plaintext: *mut signal_buffer,
    decrypt_context: *mut c_void,
) -> c_int {
    let mut callback: Rc<DecryptionCallbackProxy> =
        unsafe { Rc::from_raw(decrypt_context as *const DecryptionCallbackProxy) };

    // Turn the signal_buffer into a vec.
    let input = unsafe { (*plaintext).data_slice() }.to_vec();

    let receiver = Rc::make_mut(&mut callback).callback(input);
    match receiver.recv() {
        Ok(Ok(_)) => 0,
        _ => DecryptionError::DecryptionCallbackFailure.as_int(),
    }
}

impl SimpleObjectTracker for GroupCipher {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl GroupCipher {
    pub fn new(
        id: TrackerId,
        store_context: StoreContext,
        ctxt: &SignalContext,
        sender_key_name: SenderKeyName,
        decrypt_callback: DecryptionCallbackProxy,
    ) -> Option<Self> {
        if let Some(cipher) = FfiGroupCipher::new(
            store_context.ffi(),
            ctxt,
            &sender_key_name.group_id,
            &sender_key_name.sender.name,
            sender_key_name.sender.device_id as i32,
        ) {
            let callback = Rc::new(decrypt_callback);
            let ptr = Rc::into_raw(callback) as *mut c_void;
            let callback: Rc<DecryptionCallbackProxy> =
                unsafe { Rc::from_raw(ptr as *const DecryptionCallbackProxy) };
            let cipher = GroupCipher {
                id,
                ffi: Arc::new(cipher),
                callback,
                store_context,
            };
            cipher.ffi.set_callback(native_decrypt_callback, ptr);
            return Some(cipher);
        }
        None
    }
}

impl GroupCipherMethods for GroupCipher {
    fn decrypt(&mut self, responder: &GroupCipherDecryptResponder, ciphertext: Vec<u8>) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("decrypt".to_string())
            .spawn(move || {
                match ffi.decrypt(&ciphertext) {
                    Ok(plaintext) => {
                        responder.resolve(plaintext);
                    }
                    Err(code) => {
                        responder.reject(code.as_int() as i64);
                    }
                };
            })
            .expect("Failed to create decrypt thread");
    }

    fn encrypt(&mut self, responder: &GroupCipherEncryptResponder, padded_plaintext: Vec<u8>) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("encrypt".to_string())
            .spawn(move || {
                match ffi.encrypt(&padded_plaintext) {
                    Ok(message) => responder.resolve(message),
                    Err(code) => responder.reject(code as i64),
                };
            })
            .expect("Failed to create encrypt thread");
    }
}
