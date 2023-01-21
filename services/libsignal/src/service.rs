/// Implementation of the libsignal service.
use crate::crypto_utils::{HmacSha256, Sha512Digest};
use crate::download_decrypt::download_decrypt;
use crate::generated::common::*;
use crate::generated::service::*;
use crate::global_context::GlobalContext;
use common::core::BaseMessage;
use common::traits::{
    CommonResponder, EmptyConfig, EmptyState, ObjectTrackerMethods, OriginAttributes, Service,
    SessionSupport, Shared, SharedServiceState, SharedSessionContext, TrackerId,
};
use libsignal_sys::SignalContext;
use log::{error, info};
use parking_lot::Mutex;
use std::rc::Rc;
use std::sync::Arc;
use threadpool::ThreadPool;

pub struct SignalService {
    id: TrackerId,
    tracker: Arc<Mutex<SignalTrackerType>>,
    proxy_tracker: Arc<Mutex<SignalProxyTracker>>,
    transport: SessionSupport,
    origin_attributes: OriginAttributes,
    pool: ThreadPool,
}

impl Signal for SignalService {
    fn get_tracker(&mut self) -> Arc<Mutex<SignalTrackerType>> {
        Arc::clone(&self.tracker)
    }

    fn get_proxy_tracker(&mut self) -> Arc<Mutex<SignalProxyTracker>> {
        Arc::clone(&self.proxy_tracker)
    }
}

impl LibSignalMethods for SignalService {
    fn curve_calculate_agreement(
        &mut self,
        responder: LibSignalCurveCalculateAgreementResponder,
        public_key: Vec<u8>,
        private_key: Vec<u8>,
    ) {
        // If the public key length is 33 bytes, remove the first one.
        let mut pub_key = [0u8; 32];
        let start = usize::from(public_key.len() == 33);

        pub_key[start..public_key.len()].copy_from_slice(&public_key[start..]);

        if let Ok(res) = SignalContext::curve_calculate_agreement(&pub_key, &private_key) {
            responder.resolve(res.to_vec());
        } else {
            responder.reject();
        }
    }

    fn curve_verify_signature(
        &mut self,
        responder: LibSignalCurveVerifySignatureResponder,
        public_key: Vec<u8>,
        message: Vec<u8>,
        signature: Vec<u8>,
    ) {
        responder.resolve(SignalContext::curve_verify_signature(
            &public_key,
            &message,
            &signature,
        ));
    }

    fn download_and_decrypt(
        &mut self,
        responder: LibSignalDownloadAndDecryptResponder,
        url: String,
        iv: Vec<u8>,
        cipher_key: Vec<u8>,
        hmac_key: Vec<u8>,
        num_ciphertext_bytes: i64,
        num_tail_bytes: i64,
        callback: ObjectRef,
    ) {
        // Since this is not doing CORS check, make sure the caller has the systemXHR permission.
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "systemXHR",
            "download_and_decrypt",
        ) {
            return;
        }

        // Check that we can access the callback proxy.
        let tracker = self.proxy_tracker.lock();
        let callback_proxy = match tracker.get(&callback) {
            Some(SignalProxy::DecryptionCallback(callback)) => callback,
            _ => {
                error!("Unable to access the download_and_decrypt callback");
                responder.reject("no_callback".into());
                return;
            }
        };

        let mut thread_callback = callback_proxy.clone();

        // Running this call on its own thread since this can take a while.
        self.pool.execute(move || {
            match download_decrypt(
                &url,
                &iv,
                &cipher_key,
                &hmac_key,
                num_ciphertext_bytes,
                num_tail_bytes,
                |buf| {
                    // Send the decrypted chunk to the callback.
                    // In this case we don't care about the callback response so we don't
                    // block on getting a response.
                    thread_callback.callback(buf.to_vec());
                },
            ) {
                Ok(res) => responder.resolve(res),
                Err(err) => {
                    error!("download_and_decrypt error: {}", err);
                    responder.reject(err);
                }
            }
        });
    }

    fn start_hmac_sha256(&mut self, responder: LibSignalStartHmacSha256Responder, key: Vec<u8>) {
        let mut tracker = self.tracker.lock();
        if let Some(wrapper) = HmacSha256::new(tracker.next_id(), &key) {
            let object = Rc::new(wrapper);
            tracker.track(SignalTrackedObject::HmacSha256(object.clone()));
            responder.resolve(object);
        } else {
            error!("Failed to create HmacSha256 wrapper.");
            responder.reject();
        }
    }

    fn start_sha512_digest(&mut self, responder: LibSignalStartSha512DigestResponder) {
        let mut tracker = self.tracker.lock();
        let object = Rc::new(Sha512Digest::new(tracker.next_id()));
        tracker.track(SignalTrackedObject::Sha512Digest(object.clone()));
        responder.resolve(object);
    }

    fn new_global_context(&mut self, responder: LibSignalNewGlobalContextResponder) {
        let mut tracker = self.tracker.lock();
        if let Some(context) = GlobalContext::new(
            tracker.next_id(),
            self.id,
            Arc::clone(&self.tracker),
            Arc::clone(&self.proxy_tracker),
            self.transport.clone(),
            self.pool.clone(),
        ) {
            let object = Rc::new(context);
            tracker.track(SignalTrackedObject::GlobalContext(object.clone()));
            responder.resolve(object);
        } else {
            error!("Failed to create GlobalContext");
            responder.reject();
        }
    }
}

common::impl_shared_state!(SignalService, EmptyState, EmptyConfig);

impl Service<SignalService> for SignalService {
    fn create(
        origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        transport: SessionSupport,
    ) -> Result<SignalService, String> {
        info!("SignalService::create");

        let service_id = transport.session_tracker_id().service();
        Ok(SignalService {
            id: service_id,
            tracker: Arc::default(),
            proxy_tracker: Arc::default(),
            transport,
            origin_attributes: origin_attributes.clone(),
            pool: ThreadPool::with_name("SignalService".into(), 5),
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<SignalFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("Signal request: {:?}", req),
            Err(err) => format!("Unable to format Signal request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        error!("Releasing object {} in Signal service", object_id);
        self.proxy_tracker
            .lock()
            .remove(&object_id.into())
            .is_some()
    }
}

impl Drop for SignalService {
    fn drop(&mut self) {
        info!("Dropping Signal Service #{}", self.id);
        self.proxy_tracker.lock().clear();
    }
}
