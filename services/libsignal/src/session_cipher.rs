use crate::generated::common::*;
use crate::store_context::StoreContext;
use common::traits::{SimpleObjectTracker, TrackerId};
use libsignal_sys::ffi::{session_cipher, signal_buffer, signal_protocol_address, DecryptionError};
use libsignal_sys::{SessionCipher as FfiSessionCipher, SignalContext};
use log::debug;
use std::os::raw::{c_int, c_void};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

pub struct SessionCipher {
    id: TrackerId,
    ffi: Arc<FfiSessionCipher>,
    address: *const signal_protocol_address,
    #[allow(dead_code)] // We need to hold the callback proxy alive with the same lifetime.
    callback: Rc<DecryptionCallbackProxy>,
    #[allow(dead_code)] // We need to hold the store context alive with the same lifetime.
    store_context: StoreContext,
}

impl Drop for SessionCipher {
    fn drop(&mut self) {
        debug!("Dropping SessionCipher #{}", self.id);
        // Regain ownership of the address to drop it.
        let addr: Rc<signal_protocol_address> = unsafe { Rc::from_raw(self.address) };
        addr.destroy();
    }
}

// Returns a negative value in case of failure.
extern "C" fn native_decrypt_callback(
    _cipher: *mut session_cipher,
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

impl SimpleObjectTracker for SessionCipher {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl SessionCipher {
    pub fn new(
        id: TrackerId,
        store_context: StoreContext,
        remote_address: Address,
        ctxt: &SignalContext,
        decrypt_callback: DecryptionCallbackProxy,
    ) -> Option<Self> {
        let signal_address = Rc::new(signal_protocol_address::new(
            &remote_address.name,
            remote_address.device_id as i32,
        ));

        // Intentionnaly leak temporarily to not drop the address.
        let address = Rc::into_raw(signal_address);

        if let Some(ffi) = FfiSessionCipher::new(store_context.ffi(), address, ctxt) {
            let callback = Rc::new(decrypt_callback);
            let ptr = Rc::into_raw(callback) as *mut c_void;
            let callback: Rc<DecryptionCallbackProxy> =
                unsafe { Rc::from_raw(ptr as *const DecryptionCallbackProxy) };
            let cipher = SessionCipher {
                id,
                ffi: Arc::new(ffi),
                address,
                callback,
                store_context,
            };
            cipher.ffi.set_callback(native_decrypt_callback, ptr);
            return Some(cipher);
        }

        // If we can't create a cipher, release the address.
        let addr: Rc<signal_protocol_address> = unsafe { Rc::from_raw(address) };
        addr.destroy();

        None
    }
}

impl SessionCipherMethods for SessionCipher {
    fn decrypt_message(
        &mut self,
        responder: &SessionCipherDecryptMessageResponder,
        ciphertext: Vec<u8>,
    ) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("decrypt".to_string())
            .spawn(move || {
                match ffi.decrypt_message(&ciphertext) {
                    Ok(plaintext) => responder.resolve(plaintext),
                    Err(code) => responder.reject(code.as_int() as i64),
                };
            })
            .expect("Failed to create decrypt thread");
    }

    fn decrypt_pre_key_message(
        &mut self,
        responder: &SessionCipherDecryptPreKeyMessageResponder,
        ciphertext: Vec<u8>,
    ) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("decrypt_pre_key".to_string())
            .spawn(move || {
                match ffi.decrypt_pre_key_message(&ciphertext) {
                    Ok(plaintext) => responder.resolve(plaintext),
                    Err(code) => responder.reject(code.as_int() as i64),
                };
            })
            .expect("Failed to create decrypt_pre_key thread");
    }

    fn encrypt(&mut self, responder: &SessionCipherEncryptResponder, padded_message: Vec<u8>) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("encrypt".to_string())
            .spawn(move || {
                match ffi.encrypt(&padded_message) {
                    Ok(message) => {
                        responder.resolve(CiphertextMessage {
                            message_type: message.0 as i64,
                            serialized: message.1,
                        });
                    }
                    Err(code) => responder.reject(code as i64),
                };
            })
            .expect("Failed to create encrypt thread");
    }

    fn remote_registration_id(&mut self, responder: &SessionCipherRemoteRegistrationIdResponder) {
        let ffi = self.ffi.clone();
        let responder = responder.clone();

        thread::Builder::new()
            .name("remote_registration_id".to_string())
            .spawn(move || {
                match ffi.remote_registration_id() {
                    Some(result) => responder.resolve(result as i64),
                    None => responder.reject(),
                };
            })
            .expect("Failed to create remote_registration_id thread");
    }
}
