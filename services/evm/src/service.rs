use crate::generated::{common::*, service::*};
use crate::provider::{get_chain_url, EvmProvider};
use common::core::BaseMessage;
use common::traits::{
    EmptyConfig, EmptyState, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport,
    Shared, SharedServiceState, SharedSessionContext, TrackerId,
};
use log::{debug, info};
use std::rc::Rc;

pub struct EvmServiceImpl {
    id: TrackerId,
    tracker: EvmServiceTrackerType,
}

impl EvmService for EvmServiceImpl {
    fn get_tracker(&mut self) -> &mut EvmServiceTrackerType {
        &mut self.tracker
    }
}

impl EvmMethods for EvmServiceImpl {
    fn get_provider(&mut self, responder: EvmGetProviderResponder, chain_name: &str) {
        if let Some(url) = get_chain_url(chain_name) {
            let id = self.tracker.next_id();
            if let Some(provider) = EvmProvider::new(url, id) {
                // Track this provider.
                let tracked = Rc::new(provider);
                self.tracker
                    .track(EvmServiceTrackedObject::Provider(tracked.clone()));
                responder.resolve(tracked);
            } else {
                responder.reject(EvmError::InternalError);
            }
        } else {
            responder.reject(EvmError::NoSuchChain);
        }
    }
}

common::impl_shared_state!(EvmServiceImpl, EmptyState, EmptyConfig);

impl Service<EvmServiceImpl> for EvmServiceImpl {
    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<EvmServiceImpl, String> {
        info!("EvmServiceImpl::create");
        let service_id = helper.session_tracker_id().service();
        Ok(EvmServiceImpl {
            id: service_id,
            tracker: EvmServiceTrackerType::default(),
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<EvmServiceFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => {
                let full = format!("EvmService request: {:?}", req);
                let len = std::cmp::min(256, full.len());
                (&full[..len]).into()
            }
            Err(err) => format!("Unable to format EvmService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        // self.tracker.lock().untrack(object_id)
        true
    }
}

impl Drop for EvmServiceImpl {
    fn drop(&mut self) {
        debug!("Dropping Evm Service #{}", self.id);

        self.tracker.clear();
    }
}
