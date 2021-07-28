use crate::generated::service::{
    SignalProxy, SignalProxyTracker, SignalTrackedObject, SignalTrackerType,
};
use crate::group_session_builder::GroupSessionBuilder;
use crate::session_builder::SessionBuilder;
use crate::session_cipher::SessionCipher;
use crate::store_context::{StoreContext as ImplStoreContext, StoreProxies};
use crate::{generated::common::*, group_cipher};
use common::traits::{ObjectTrackerMethods, SessionSupport, SimpleObjectTracker, TrackerId};
use group_cipher::GroupCipher;
use libsignal_sys::SignalContext;
use log::error;
use parking_lot::Mutex;
use std::rc::Rc;
use std::sync::Arc;
use threadpool::ThreadPool;

pub struct GlobalContext {
    id: TrackerId,
    service_id: TrackerId,
    signal_context: SignalContext,
    tracker: Arc<Mutex<SignalTrackerType>>,
    proxy_tracker: Arc<Mutex<SignalProxyTracker>>,
    transport: SessionSupport,
    pool: ThreadPool,
}

impl GlobalContext {
    pub fn new(
        id: TrackerId,
        service_id: TrackerId,
        tracker: Arc<Mutex<SignalTrackerType>>,
        proxy_tracker: Arc<Mutex<SignalProxyTracker>>,
        transport: SessionSupport,
        pool: ThreadPool,
    ) -> Option<Self> {
        SignalContext::new().map(|signal_context| GlobalContext {
            id,
            service_id,
            signal_context,
            tracker,
            proxy_tracker,
            transport,
            pool,
        })
    }
}

impl SimpleObjectTracker for GlobalContext {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl GlobalContext {
    fn maybe_add_proxy<F>(&mut self, object_ref: ObjectRef, builder: F)
    where
        F: FnOnce() -> SignalProxy,
    {
        let mut lock = self.proxy_tracker.lock();
        lock.entry(object_ref).or_insert_with(builder);
    }

    // TODO: fix the codegen so we don't have to do it manually.
    fn maybe_add_decrypt_callback(&mut self, callback_ref: ObjectRef) {
        let service_id = self.service_id;
        let transport = self.transport.clone();

        self.maybe_add_proxy(callback_ref, || {
            SignalProxy::DecryptionCallback(DecryptionCallbackProxy::new(
                callback_ref,
                service_id,
                &transport,
            ))
        })
    }

    fn maybe_store_proxies(&mut self, store: &StoreContext) -> Option<StoreProxies> {
        let service_id = self.service_id;
        let transport = self.transport.clone();

        // Make sure the store context callbacks are registered in the proxy tracker.
        // This is not done yet by codegen because they are not direct parameters.
        self.maybe_add_proxy(store.session_store, || {
            SignalProxy::SessionStore(SessionStoreProxy::new(
                store.session_store,
                service_id,
                &transport,
            ))
        });
        self.maybe_add_proxy(store.identity_key_store, || {
            SignalProxy::IdentityKeyStore(IdentityKeyStoreProxy::new(
                store.identity_key_store,
                service_id,
                &transport,
            ))
        });
        self.maybe_add_proxy(store.pre_key_store, || {
            SignalProxy::KeyStore(KeyStoreProxy::new(
                store.pre_key_store,
                service_id,
                &transport,
            ))
        });
        self.maybe_add_proxy(store.signed_pre_key_store, || {
            SignalProxy::KeyStore(KeyStoreProxy::new(
                store.signed_pre_key_store,
                service_id,
                &transport,
            ))
        });
        self.maybe_add_proxy(store.sender_key_store, || {
            SignalProxy::SenderKeyStore(SenderKeyStoreProxy::new(
                store.sender_key_store,
                service_id,
                &transport,
            ))
        });

        let tracker = self.proxy_tracker.lock();

        let session_store: Option<&SignalProxy> = tracker.get(&store.session_store);
        let identity_key_store: Option<&SignalProxy> = tracker.get(&store.identity_key_store);
        let pre_key_store: Option<&SignalProxy> = tracker.get(&store.pre_key_store);
        let signed_pre_key_store: Option<&SignalProxy> = tracker.get(&store.signed_pre_key_store);
        let sender_key_store: Option<&SignalProxy> = tracker.get(&store.sender_key_store);

        if let (
            Some(SignalProxy::SessionStore(session_store)),
            Some(SignalProxy::IdentityKeyStore(identity_key_store)),
            Some(SignalProxy::KeyStore(pre_key_store)),
            Some(SignalProxy::KeyStore(signed_pre_key_store)),
            Some(SignalProxy::SenderKeyStore(sender_key_store)),
        ) = (
            session_store,
            identity_key_store,
            pre_key_store,
            signed_pre_key_store,
            sender_key_store,
        ) {
            return Some(StoreProxies {
                session_store: session_store.clone(),
                identity_key_store: identity_key_store.clone(),
                pre_key_store: pre_key_store.clone(),
                signed_pre_key_store: signed_pre_key_store.clone(),
                sender_key_store: sender_key_store.clone(),
            });
        }

        error!("Failed to get all store proxies!");
        None
    }
}

impl GlobalContextMethods for GlobalContext {
    fn generate_identity_key_pair(
        &mut self,
        responder: GlobalContextGenerateIdentityKeyPairResponder,
    ) {
        if let Ok((public_key, private_key)) = self.signal_context.generate_identity_key_pair() {
            responder.resolve(RatchetIdentityKeyPair {
                public_key: public_key.to_vec(),
                private_key: private_key.to_vec(),
            });
        } else {
            responder.reject();
        }
    }

    fn generate_pre_keys(
        &mut self,
        responder: GlobalContextGeneratePreKeysResponder,
        start: i64,
        count: i64,
    ) {
        if let Ok(keys) = self
            .signal_context
            .generate_pre_keys(start as u32, count as u32)
        {
            let mut res = vec![];
            for key in keys.iter() {
                res.push(SessionPreKey {
                    id: key.0 as i64,
                    key_pair: EcKeyPair {
                        public_key: key.1.to_vec(),
                        private_key: key.2.to_vec(),
                    },
                });
            }

            responder.resolve(res);
        } else {
            responder.reject();
        }
    }

    fn generate_registration_id(
        &mut self,
        responder: GlobalContextGenerateRegistrationIdResponder,
        extended_range: bool,
    ) {
        if let Ok(reg_id) = self
            .signal_context
            .get_registration_id(extended_range)
            .map(|v| v as i64)
        {
            responder.resolve(reg_id);
        } else {
            responder.reject();
        }
    }

    fn generate_sender_key(&mut self, responder: GlobalContextGenerateSenderKeyResponder) {
        if let Ok(key) = self.signal_context.generate_sender_key() {
            responder.resolve(key);
        } else {
            responder.reject();
        }
    }

    fn generate_sender_key_id(&mut self, responder: GlobalContextGenerateSenderKeyIdResponder) {
        if let Ok(key_id) = self
            .signal_context
            .generate_sender_key_id()
            .map(|val| val as i64)
        {
            responder.resolve(key_id);
        } else {
            responder.reject();
        }
    }

    fn generate_sender_signing_key(
        &mut self,
        responder: GlobalContextGenerateSenderSigningKeyResponder,
    ) {
        if let Ok(key_pair) = self.signal_context.generate_sender_signing_key() {
            responder.resolve(EcKeyPair {
                public_key: key_pair.0.to_vec(),
                private_key: key_pair.1.to_vec(),
            });
        } else {
            responder.reject();
        }
    }

    fn generate_signed_pre_key(
        &mut self,
        responder: GlobalContextGenerateSignedPreKeyResponder,
        identity_key_pair: RatchetIdentityKeyPair,
        signed_pre_key_id: i64,
        timestamp: i64,
    ) {
        if let Ok(key) = self.signal_context.generate_signed_pre_key(
            &identity_key_pair.public_key,
            &identity_key_pair.private_key,
            signed_pre_key_id as u32,
            timestamp as u64,
        ) {
            // -> (id, public_key, private_key, timestamp, signature)
            let res = SessionSignedPreKey {
                id: key.0 as i64,
                timestamp: key.3 as i64,
                signature: key.4,
                key_pair: EcKeyPair {
                    public_key: key.1.to_vec(),
                    private_key: key.2.to_vec(),
                },
            };

            responder.resolve(res);
        } else {
            responder.reject();
        }
    }

    fn group_cipher(
        &mut self,
        responder: GlobalContextGroupCipherResponder,
        store_context: StoreContext,
        sender_key_name: SenderKeyName,
        callback: ObjectRef,
    ) {
        // Fine because we know we didn't jump thread.
        if self.tracker.is_locked() {
            unsafe {
                self.tracker.force_unlock();
            }
        }

        self.maybe_add_decrypt_callback(callback);

        let mut success = false;
        if let Some(proxies) = self.maybe_store_proxies(&store_context) {
            if let Some(store_context) =
                ImplStoreContext::new(&self.signal_context, Rc::new(proxies))
            {
                if let Some(SignalProxy::DecryptionCallback(proxy)) =
                    self.proxy_tracker.lock().get(&callback)
                {
                    let mut tracker = self.tracker.lock();
                    if let Some(group_cipher) = GroupCipher::new(
                        tracker.next_id(),
                        store_context,
                        &self.signal_context,
                        sender_key_name,
                        proxy.clone(),
                        self.pool.clone(),
                    ) {
                        let object = Rc::new(group_cipher);
                        tracker.track(SignalTrackedObject::GroupCipher(object.clone()));
                        success = true;
                        responder.resolve(object);
                    }
                }
            }
        }

        if !success {
            // Fallback on any error.
            responder.reject();
        }
    }

    fn group_session_builder(
        &mut self,
        responder: GlobalContextGroupSessionBuilderResponder,
        store_context: StoreContext,
    ) {
        // Fine because we know we didn't jump thread.
        if self.tracker.is_locked() {
            unsafe {
                self.tracker.force_unlock();
            }
        }
        let mut success = false;
        if let Some(proxies) = self.maybe_store_proxies(&store_context) {
            if let Some(store_context) =
                ImplStoreContext::new(&self.signal_context, Rc::new(proxies))
            {
                let mut tracker = self.tracker.lock();
                if let Some(session_builder) = GroupSessionBuilder::new(
                    store_context,
                    &self.signal_context,
                    tracker.next_id(),
                    self.pool.clone(),
                ) {
                    let object = Rc::new(session_builder);
                    tracker.track(SignalTrackedObject::GroupSessionBuilder(object.clone()));
                    success = true;
                    responder.resolve(object);
                }
            }
        }

        if !success {
            // Fallback on any error.
            responder.reject();
        }
    }

    fn session_builder(
        &mut self,
        responder: GlobalContextSessionBuilderResponder,
        address: Address,
        store_context: StoreContext,
    ) {
        // Fine because we know we didn't jump thread.
        if self.tracker.is_locked() {
            unsafe {
                self.tracker.force_unlock();
            }
        }
        let mut success = false;
        if let Some(proxies) = self.maybe_store_proxies(&store_context) {
            if let Some(store_context) =
                ImplStoreContext::new(&self.signal_context, Rc::new(proxies))
            {
                let mut tracker = self.tracker.lock();
                if let Some(session_builder) = SessionBuilder::new(
                    store_context,
                    address,
                    &self.signal_context,
                    tracker.next_id(),
                    self.pool.clone(),
                ) {
                    let object = Rc::new(session_builder);
                    tracker.track(SignalTrackedObject::SessionBuilder(object.clone()));
                    success = true;
                    responder.resolve(object);
                }
            }
        }

        if !success {
            // Fallback on any error.
            responder.reject();
        }
    }

    fn session_cipher(
        &mut self,
        responder: GlobalContextSessionCipherResponder,
        address: Address,
        store_context: StoreContext,
        callback: ObjectRef,
    ) {
        // Fine because we know we didn't jump thread.
        if self.tracker.is_locked() {
            unsafe {
                self.tracker.force_unlock();
            }
        }

        self.maybe_add_decrypt_callback(callback);

        let mut success = false;
        if let Some(proxies) = self.maybe_store_proxies(&store_context) {
            if let Some(store_context) =
                ImplStoreContext::new(&self.signal_context, Rc::new(proxies))
            {
                if let Some(SignalProxy::DecryptionCallback(proxy)) =
                    self.proxy_tracker.lock().get(&callback)
                {
                    let mut tracker = self.tracker.lock();
                    if let Some(session_cipher) = SessionCipher::new(
                        tracker.next_id(),
                        store_context,
                        address,
                        &self.signal_context,
                        proxy.clone(),
                        self.pool.clone(),
                    ) {
                        let object = Rc::new(session_cipher);
                        tracker.track(SignalTrackedObject::SessionCipher(object.clone()));
                        success = true;
                        responder.resolve(object);
                    }
                }
            }
        }

        if !success {
            // Fallback on any error.
            responder.reject();
        }
    }
}
