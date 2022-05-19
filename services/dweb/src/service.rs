use crate::config::Config;
use crate::did::Did;
use crate::generated::common::Did as SidlDid;
use crate::generated::{common::*, service::*};
use crate::storage::DidStorage;
use async_std::path::Path;
use common::core::BaseMessage;
use common::traits::{
    DispatcherId, OriginAttributes, Service, SessionSupport, Shared, SharedServiceState,
    SharedSessionContext, StateLogger, TrackerId,
};
use log::{debug, info};

pub struct State {
    did_store: DidStorage,
    event_broadcaster: DwebEventBroadcaster,
}

impl StateLogger for State {}

impl Into<State> for &Config {
    fn into(self) -> State {
        let config_path = self.storage_path();
        let store_path = Path::new(&config_path);

        State {
            did_store: DidStorage::new(store_path),
            event_broadcaster: DwebEventBroadcaster::default(),
        }
    }
}

pub struct DWebServiceImpl {
    id: TrackerId,
    state: Shared<State>,
    dispatcher_id: DispatcherId,
    _origin_attributes: OriginAttributes,
}

impl DwebService for DWebServiceImpl {
    // fn get_tracker(&mut self) -> Arc<Mutex<DWebTrackerType>> {
    //     self.tracker.clone()
    // }

    // fn get_proxy_tracker(&mut self) -> &mut DWebProxyTracker {
    //     &mut self.proxy_tracker
    // }
}

impl DWebServiceImpl {}

impl DwebMethods for DWebServiceImpl {
    fn create_did(&mut self, responder: DwebCreateDidResponder, name: String) {
        let did = Did::create(&name);
        let mut state = self.state.lock();
        if state.did_store.add(&did) {
            if let Err(_) = state.did_store.save() {
                responder.reject(DidError::InternalError);
            } else {
                let sdid: SidlDid = did.into();
                state.event_broadcaster.broadcast_didcreated(sdid.clone());
                responder.resolve(sdid);
            }
        } else {
            responder.reject(DidError::InternalError);
        }
    }

    fn get_dids(&mut self, responder: DwebGetDidsResponder) {
        let result = self.state.lock().did_store.get_all();
        responder.resolve(Some(result));
    }

    fn remove_did(&mut self, responder: DwebRemoveDidResponder, uri: String) {
        let mut state = self.state.lock();
        if state.did_store.remove(&uri) {
            state.event_broadcaster.broadcast_didremoved(uri);
            responder.resolve();
        } else {
            responder.reject(DidError::UnknownDid);
        }
    }
}

common::impl_shared_state!(DWebServiceImpl, State, Config);

impl Service<DWebServiceImpl> for DWebServiceImpl {
    fn create(
        attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<DWebServiceImpl, String> {
        info!("DWebServiceImpl::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = DwebEventDispatcher::from(helper, 0 /* object id */);
        let state = Self::shared_state();
        let dispatcher_id = state.lock().event_broadcaster.add(&event_dispatcher);
        Ok(DWebServiceImpl {
            id: service_id,
            state,
            dispatcher_id,
            _origin_attributes: attrs.clone(),
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<DwebServiceFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => {
                let full = format!("DWebService request: {:?}", req);
                let len = std::cmp::min(256, full.len());
                (&full[..len]).into()
            }
            Err(err) => format!("Unable to format DWebService request: {:?}", err),
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

impl Drop for DWebServiceImpl {
    fn drop(&mut self) {
        debug!("Dropping DWeb Service #{}", self.id);
        let state = &mut self.state.lock();

        let dispatcher_id = self.dispatcher_id;
        state.event_broadcaster.remove(dispatcher_id);

        // TODO:
        // self.observers.clear(...);

        // self.tracker.lock().clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common::core::{BaseMessage, BaseMessageKind};
    use common::traits::*;
    use std::collections::HashSet;

    use crate::config::Config;
    use crate::service::DWebServiceImpl;

    fn base_message() -> BaseMessage {
        BaseMessage {
            service: 0,
            object: 0,
            kind: BaseMessageKind::Response(0),
            content: vec![],
        }
    }

    #[test]
    fn service_creation() {
        let session_context = SessionContext::default();
        let (sender, _receiver) = std::sync::mpsc::channel();
        let shared_sender = MessageSender::new(Box::new(StdSender::new(&sender)));

        let helper = SessionSupport::new(
            SessionTrackerId::from(0, 0),
            shared_sender,
            Shared::adopt(IdFactory::new(0)),
            Shared::default(),
        );

        DWebServiceImpl::init_shared_state(&Config::new("./test-content"));

        let mut service: DWebServiceImpl = DWebServiceImpl::create(
            &OriginAttributes::new("test", HashSet::new()),
            Shared::adopt(session_context),
            helper.clone(),
        )
        .unwrap();

        let responder = DwebCreateDidResponder::new(helper, base_message());
        service.create_did(responder, "test me".into());
    }
}
