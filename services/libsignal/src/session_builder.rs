use crate::generated::common::*;
use crate::store_context::StoreContext;
use common::traits::{SimpleObjectTracker, TrackerId};
use libsignal_sys::ffi::signal_protocol_address;
use libsignal_sys::{
    SessionBuilder as FfiSessionBuilder, SessionPreKeyBundle as FfiSessionPreKeyBundle,
    SignalContext,
};
use log::{debug, error};
use std::rc::Rc;
use std::sync::Arc;
use threadpool::ThreadPool;

pub struct SessionBuilder {
    id: TrackerId,
    ffi: Arc<FfiSessionBuilder>,
    address: *const signal_protocol_address,
    #[allow(dead_code)] // We need to hold the store context alive with the same lifetime.
    store_context: StoreContext,
    pool: ThreadPool,
}

impl Drop for SessionBuilder {
    fn drop(&mut self) {
        debug!("Dropping SessionBuilder #{}", self.id);
        // Regain ownership of the address to drop it.
        let addr: Rc<signal_protocol_address> = unsafe { Rc::from_raw(self.address) };
        addr.destroy();
    }
}

impl SessionBuilder {
    pub fn new(
        store_context: StoreContext,
        remote_address: Address,
        ctxt: &SignalContext,
        id: TrackerId,
        pool: ThreadPool,
    ) -> Option<Self> {
        let signal_address = Rc::new(signal_protocol_address::new(
            &remote_address.name,
            remote_address.device_id as i32,
        ));
        // Intentionnaly leak temporarily to not drop the address.
        let address = Rc::into_raw(signal_address);

        if let Some(builder) = FfiSessionBuilder::new(store_context.ffi(), address, ctxt) {
            return Some(SessionBuilder {
                id,
                ffi: Arc::new(builder),
                address,
                store_context,
                pool,
            });
        }

        // If we can't create a cipher, release the address.
        let addr: Rc<signal_protocol_address> = unsafe { Rc::from_raw(address) };
        addr.destroy();
        None
    }
}

impl SimpleObjectTracker for SessionBuilder {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl SessionBuilderMethods for SessionBuilder {
    fn process_pre_key_bundle(
        &mut self,
        responder: SessionBuilderProcessPreKeyBundleResponder,
        bundle: SessionPreKeyBundle,
    ) {
        let ffi = self.ffi.clone();
        self.pool.execute(move || {
            fn key_from_slice(bytes: &[u8]) -> [u8; 32] {
                let mut a = [0; 32];
                a.clone_from_slice(bytes);
                a
            }

            let key: [u8; 32];
            let prekey_pub = match bundle.pre_key_public.len() {
                32 => {
                    key = key_from_slice(&bundle.pre_key_public);
                    Some(&key)
                }
                _ => None,
            };

            if let Some(ffi_bundle) = FfiSessionPreKeyBundle::new(
                bundle.registration_id as u32,
                bundle.device_id as u32,
                bundle.pre_key_id as u32,
                prekey_pub,
                bundle.signed_pre_key_id as u32,
                &key_from_slice(&bundle.signed_pre_key_public),
                &bundle.signed_pre_key_signature,
                &key_from_slice(&bundle.identity_key),
            ) {
                if ffi.process_pre_key_bundle(&ffi_bundle) {
                    responder.resolve();
                } else {
                    error!("ffi.process_pre_key_bundle returned false");
                    responder.reject();
                }
            }
        });
    }
}
