use crate::config::Config;
use crate::did::Did;
use crate::generated::common::Did as SidlDid;
use crate::generated::{common::*, service::*};
use crate::storage::DwebStorage;
use async_std::path::Path;
use common::core::BaseMessage;
use common::traits::{
    CommonResponder, DispatcherId, OriginAttributes, Service, SessionSupport, Shared,
    SharedServiceState, SharedSessionContext, StateLogger, TrackerId,
};
use log::{debug, error, info};
use std::rc::Rc;
use std::time::SystemTime;
use ucan::builder::UcanBuilder;
use ucan::capability::{Action, Capability as UCapability, Resource, Scope, With};
use ucan::ucan::Ucan;
use url::Url as StdUrl;

pub struct State {
    pub dweb_store: DwebStorage,
    event_broadcaster: DwebEventBroadcaster,
    ui_provider: Option<UcanProviderProxy>,
}

impl StateLogger for State {}

#[allow(clippy::from_over_into)]
impl Into<State> for &Config {
    fn into(self) -> State {
        let config_path = self.storage_path();
        let store_path = Path::new(&config_path);

        State {
            dweb_store: DwebStorage::new(store_path),
            event_broadcaster: DwebEventBroadcaster::default(),
            ui_provider: None,
        }
    }
}

#[derive(Clone, PartialEq)]
struct UrlScope(StdUrl);

impl ToString for UrlScope {
    fn to_string(&self) -> std::string::String {
        format!("{}", self.0)
    }
}

impl From<StdUrl> for UrlScope {
    fn from(url: StdUrl) -> Self {
        UrlScope(url)
    }
}

impl Scope for UrlScope {
    // TODO: implement properly if we need it.
    fn contains(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
struct ActionString(String);

impl ToString for ActionString {
    fn to_string(&self) -> std::string::String {
        self.0.clone()
    }
}

impl From<String> for ActionString {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl Action for ActionString {}

fn as_ucan_capability(capability: Capability) -> UCapability<UrlScope, ActionString> where
{
    UCapability::new(
        With::Resource {
            kind: Resource::Scoped(UrlScope(capability.scope)),
        },
        ActionString(capability.action),
    )
}

pub struct DWebServiceImpl {
    id: TrackerId,
    state: Shared<State>,
    dispatcher_id: DispatcherId,
    origin_attributes: OriginAttributes,
    proxy_tracker: DwebServiceProxyTracker,
    provides_ui: bool,
    tracker: DwebServiceTrackerType,
}

impl DwebService for DWebServiceImpl {
    fn get_tracker(&mut self) -> &mut DwebServiceTrackerType {
        &mut self.tracker
    }

    fn get_proxy_tracker(&mut self) -> &mut DwebServiceProxyTracker {
        &mut self.proxy_tracker
    }
}

fn build_ucan_token(
    state: Shared<State>,
    audience: &str,
    granted: GrantedCapabilities,
) -> Result<Ucan, ()> {
    let issuer = state
        .lock()
        .dweb_store
        .did_by_name(&granted.issuer.name)
        .map_err(|_| ())?
        .ok_or(())?
        .clone();

    let not_before = granted
        .not_before
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| ())?
        .as_secs();
    let expiration = granted
        .expiration
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| ())?
        .as_secs();

    let ucan_key = issuer.as_ucan_key();
    let mut ucan = UcanBuilder::default()
        .issued_by(&ucan_key)
        .for_audience(audience)
        .not_before(not_before)
        .with_expiration(expiration);

    if let Some(capabilities) = granted.capabilities {
        for capability in capabilities {
            ucan = ucan.claiming_capability(&as_ucan_capability(capability));
        }
    }

    let signable = ucan.build().map_err(|_| ())?;

    async_std::task::block_on(async { signable.sign().await.map_err(|_| ()) })
}

impl DwebMethods for DWebServiceImpl {
    fn create_did(&mut self, responder: DwebCreateDidResponder, name: String) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "create DID") {
            return;
        }

        let did = Did::create(&name);
        let mut state = self.state.lock();
        if let Ok(true) = state.dweb_store.add_did(&did) {
            let sdid: SidlDid = did.into();
            state.event_broadcaster.broadcast_didcreated(sdid.clone());
            responder.resolve(sdid);
        } else {
            responder.reject(DidError::InternalError);
        }
    }

    fn get_dids(&mut self, responder: DwebGetDidsResponder) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "get DIDs") {
            return;
        }

        match self.state.lock().dweb_store.get_all_dids() {
            Ok(list) => responder.resolve(Some(list)),
            Err(_) => responder.reject(DidError::InternalError),
        }
    }

    fn remove_did(&mut self, responder: DwebRemoveDidResponder, uri: String) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "remove DID") {
            return;
        }

        let mut state = self.state.lock();
        if let Ok(true) = state.dweb_store.remove_did(&uri) {
            state.event_broadcaster.broadcast_didremoved(uri);
            responder.resolve();
        } else {
            responder.reject(DidError::UnknownDid);
        }
    }

    fn set_ucan_ui(&mut self, responder: DwebSetUcanUiResponder, provider: ObjectRef) {
        // Check that the dweb permission was granted.
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "setting UCAN UI")
        {
            return;
        }

        // Check that we don't already have a UI provider setup.
        let mut state = self.state.lock();
        if state.ui_provider.is_some() {
            error!(
                "Trying to set a duplicate UCAN ui provider from {}",
                self.origin_attributes.identity()
            );
            responder.reject();
        }

        // Register the new provider.
        if let Some(DwebServiceProxy::UcanProvider(ui_proxy)) = self.proxy_tracker.get(&provider) {
            state.ui_provider = Some(ui_proxy.clone());
        } else {
            responder.reject();
        }

        self.provides_ui = true;
        responder.resolve();
    }

    fn request_capabilities(
        &mut self,
        responder: DwebRequestCapabilitiesResponder,
        audience: String,
        capabilities: Vec<Capability>,
    ) {
        let mut provider = match &self.state.lock().ui_provider {
            Some(provider) => provider.clone(),
            None => {
                responder.reject(UcanError::NoUiProvider);
                return;
            }
        };

        // Check that the audience is a valid DID uri.
        if did_key::resolve(&audience).is_err() {
            responder.reject(UcanError::InvalidAudience);
            return;
        }

        let url = self.origin_attributes.identity();
        let state = self.state.clone();
        let _ = std::thread::spawn(move || {
            // Call the provider and relay the result to this responder.
            let requested = RequestedCapabilities {
                url: url.parse().unwrap(),
                audience: audience.clone(),
                capabilities,
            };

            if let Ok(result) = provider.grant_capabilities(requested).recv() {
                match result {
                    Ok(granted) => {
                        // Build the token.
                        if let Ok(ucan) = build_ucan_token(state.clone(), &audience, granted) {
                            // Store the UCAN in the unblocked state.
                            match state.lock().dweb_store.add_ucan(&ucan, &url, false) {
                                Ok(false) | Err(_) => responder.reject(UcanError::InternalError),
                                _ => {}
                            }

                            if let Ok(base64) = ucan.encode() {
                                responder.resolve(base64);
                            } else {
                                responder.reject(UcanError::InternalError);
                            }
                        } else {
                            responder.reject(UcanError::InternalError);
                        }
                    }
                    Err(_) => responder.reject(UcanError::UiCancel),
                }
            } else {
                responder.reject(UcanError::InternalError)
            }
        });
    }

    fn request_superuser(&mut self, responder: DwebRequestSuperuserResponder) {
        // Check that the dweb permission was granted.
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "request super user",
        ) {
            return;
        }

        let maybe_audience = self.state.lock().dweb_store.did_by_name("superuser");

        if let Ok(Some(audience)) = maybe_audience {
            let granted = GrantedCapabilities {
                issuer: audience.clone().into(),
                capabilities: Some(vec![Capability {
                    scope: StdUrl::parse("my:*").unwrap().into(),
                    action: "*".into(),
                }]),
                not_before: SystemTime::now().into(),
                expiration: SystemTime::now()
                    .checked_add(std::time::Duration::from_secs(3600 * 24 * 60)) // Arbitrary 2 months validity.
                    .unwrap()
                    .into(),
            };
            if let Ok(ucan) = build_ucan_token(self.state.clone(), &audience.uri(), granted) {
                if let Ok(base64) = ucan.encode() {
                    responder.resolve(base64);
                } else {
                    responder.reject(UcanError::InternalError);
                }
            } else {
                responder.reject(UcanError::InternalError);
            }
        } else {
            responder.reject(UcanError::InternalError);
        }
    }

    fn ucans_for(&mut self, responder: DwebUcansForResponder, origin: String) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "UCANs for origin",
        ) {
            return;
        }

        if let Ok(tokens) = self.state.lock().dweb_store.ucans_for_origin(&origin) {
            let mut ucans: Vec<Box<dyn UcanMethods>> = vec![];
            for token in tokens {
                if let Some(ucan) = crate::sidl_ucan::SidlUcan::try_new(token, self.state.clone()) {
                    ucans.push(Box::new(ucan));
                }
            }
            responder.resolve(Rc::new(ucans));
        } else {
            responder.reject(UcanError::InternalError);
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
        let dispatcher_id = if attrs.has_permission("dweb") {
            state.lock().event_broadcaster.add(&event_dispatcher)
        } else {
            0
        };
        Ok(DWebServiceImpl {
            id: service_id,
            state,
            dispatcher_id,
            origin_attributes: attrs.clone(),
            proxy_tracker: DwebServiceProxyTracker::default(),
            provides_ui: false,
            tracker: DwebServiceTrackerType::default(),
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

        if self.origin_attributes.has_permission("dweb") {
            let state = &mut self.state.lock();

            let dispatcher_id = self.dispatcher_id;
            state.event_broadcaster.remove(dispatcher_id);
        }

        if self.provides_ui {
            self.state.lock().ui_provider = None;
        }

        // TODO: when needed
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
