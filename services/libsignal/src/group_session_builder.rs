use crate::generated::common::*;
use crate::store_context::StoreContext;
use common::traits::{SimpleObjectTracker, TrackerId};
use libsignal_sys::{
    GroupSessionBuilder as FfiGroupSessionBuilder,
    SenderKeyDistributionMessage as FfiSenderKeyDistributionMessage, SignalContext,
};
use log::{debug, error};
use std::sync::Arc;
use threadpool::ThreadPool;

pub struct GroupSessionBuilder {
    id: TrackerId,
    ffi: Arc<FfiGroupSessionBuilder>,
    #[allow(dead_code)] // We need to hold the store context alive with the same lifetime.
    store_context: StoreContext,
    pool: ThreadPool,
}

impl Drop for GroupSessionBuilder {
    fn drop(&mut self) {
        debug!("Dropping GroupSessionBuilder #{}", self.id);
    }
}

impl GroupSessionBuilder {
    pub fn new(
        store_context: StoreContext,
        ctxt: &SignalContext,
        id: TrackerId,
        pool: ThreadPool,
    ) -> Option<Self> {
        if let Some(builder) = FfiGroupSessionBuilder::new(store_context.ffi(), ctxt) {
            return Some(GroupSessionBuilder {
                id,
                ffi: Arc::new(builder),
                store_context,
                pool,
            });
        }
        None
    }
}

impl SimpleObjectTracker for GroupSessionBuilder {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl GroupSessionBuilderMethods for GroupSessionBuilder {
    fn create_session(
        &mut self,
        responder: GroupSessionBuilderCreateSessionResponder,
        sender_key_name: SenderKeyName,
    ) {
        let ffi = self.ffi.clone();
        self.pool.execute(move || {
            let addr = sender_key_name.sender;

            if let Ok(key_message) =
                ffi.create_session(&sender_key_name.group_id, &addr.name, addr.device_id as i32)
            {
                let message = SenderKeyDistributionMessage {
                    serialized: key_message.serialize(),
                };
                responder.resolve(message);
            } else {
                error!("ffi.create_session failed");
                responder.reject();
            }
        });
    }

    fn process_session(
        &mut self,
        responder: GroupSessionBuilderProcessSessionResponder,
        sender_key_name: SenderKeyName,
        distribution_message: SenderKeyDistributionMessage,
    ) {
        // This request will trigger synchronous callbacks from libsignal, so we run it in a thread
        // to return asap to the event loop.
        let ffi = self.ffi.clone();
        self.pool.execute(move || {
            let addr = sender_key_name.sender;
            // FfiSenderKeyDistributionMessage::deserialize is an unsafe function.
            // See safety in the implementation.
            if unsafe { ffi
                .process_session(
                    &sender_key_name.group_id,
                    &addr.name,
                    addr.device_id as i32,
                    &FfiSenderKeyDistributionMessage::deserialize(
                        &distribution_message.serialized,
                        ffi.global_context(),
                    )
                    .unwrap(),
                )
                .is_ok()
            }{
                responder.resolve();
            } else {
                error!("ffi.processs_session failed");
                responder.reject();
            }
        });
    }
}
