use crate::config::Config;
use crate::did::Did;
use crate::generated::common::Did as SidlDid;
use crate::generated::{common::*, service::*};
use crate::handshake::{HandshakeClient, Status};
use crate::mdns::MdnsDiscovery;
use crate::storage::DwebStorage;
use crate::DiscoveryMechanism;
use async_std::path::Path;
use common::core::BaseMessage;
use common::traits::{
    CommonResponder, DispatcherId, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport,
    Shared, SharedServiceState, SharedSessionContext, StateLogger, TrackerId,
};
use common::JsonValue;
use log::{debug, error, info};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::SystemTime;
use ucan::builder::UcanBuilder;
use ucan::capability::{Action, Capability as UCapability, Resource, Scope, With};
use ucan::ucan::Ucan;
use url::Url as StdUrl;

// The internal representation of known peers needs to include
// the endpoint used to communicate with them, and wether it is
// a remote or local peer.

pub struct KnownPeer {
    pub peer: Peer,
    pub is_local: bool,
    pub endpoint: SocketAddr,
    pub session_id: Option<String>,
}

fn new_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub struct State {
    pub dweb_store: DwebStorage,
    event_broadcaster: DwebEventBroadcaster,
    ui_provider: Option<UcanProviderProxy>,
    p2p_provider: Option<P2pProviderProxy>,
    known_peers: BTreeMap<String, KnownPeer>, // The key is the device id.
    mdns: Option<MdnsDiscovery>,
    sessions: BTreeMap<String, Session>, // The key is the session id.
}

impl StateLogger for State {}

impl State {
    pub fn on_peer_found(&mut self, peer: KnownPeer) {
        let key = peer.peer.device_id.clone();
        if !self.known_peers.contains_key(&key) {
            info!("Peer added: {}", key);
            self.event_broadcaster
                .broadcast_peerfound(peer.peer.clone());
            self.known_peers.insert(key, peer);
        }
    }

    pub fn on_peer_lost(&mut self, id: &str) {
        info!("Removing peer: {}", id);

        if let Some(peer) = self.known_peers.remove(id) {
            self.event_broadcaster.broadcast_peerlost(peer.peer.clone());
            self.maybe_remove_session(peer.peer);
        } else {
            error!("Failed to remove peer {}", id);
        }
    }

    pub fn get_p2p_provider(&self) -> &Option<P2pProviderProxy> {
        &self.p2p_provider
    }

    fn maybe_remove_session(&mut self, peer: Peer) {
        let mut session_id = None;
        self.sessions.retain(|_key, session| {
            let found = session.peer.did == peer.did && session.peer.device_id == peer.device_id;
            if found {
                session_id = Some(session.id.clone());
            }
            !found
        });

        if let Some(id) = session_id {
            self.event_broadcaster.broadcast_sessionremoved(id);
        }
    }

    fn create_session(&mut self, peer: Peer) -> Session {
        let session = Session {
            id: new_session_id(),
            peer,
        };

        self.sessions.insert(session.id.clone(), session.clone());
        self.event_broadcaster
            .broadcast_sessionadded(session.clone());

        session
    }
}

#[allow(clippy::from_over_into)]
impl Into<State> for &Config {
    fn into(self) -> State {
        let config_path = self.storage_path();
        let store_path = Path::new(&config_path);

        State {
            dweb_store: DwebStorage::new(store_path),
            event_broadcaster: DwebEventBroadcaster::default(),
            ui_provider: None,
            p2p_provider: None,
            known_peers: BTreeMap::new(),
            mdns: None,
            sessions: BTreeMap::new(),
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
    provides_p2p: bool,
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
            let mut ucans: Vec<Rc<dyn UcanMethods>> = vec![];
            for token in tokens {
                let id = self.tracker.next_id();
                if let Some(ucan) =
                    crate::sidl_ucan::SidlUcan::try_new(id, token, self.state.clone())
                {
                    let tracked = Rc::new(ucan);
                    self.tracker
                        .track(DwebServiceTrackedObject::Ucan(tracked.clone()));
                    ucans.push(tracked);
                }
            }
            responder.resolve(Rc::new(ucans));
        } else {
            responder.reject(UcanError::InternalError);
        }
    }

    // Peer discovery
    fn enable_discovery(
        &mut self,
        responder: DwebEnableDiscoveryResponder,
        local_only: bool,
        peer: Peer,
    ) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "enable discovery",
        ) {
            return;
        }

        let mut state = self.state.lock();

        if state.mdns.is_none() {
            state.mdns = MdnsDiscovery::with_state(self.state.clone());
        }

        if let Some(mdns) = &mut state.mdns {
            if mdns.start(&peer).is_err() {
                responder.reject();
            }
        } else {
            responder.reject();
        }

        // Start the rendez-vous discovery client.
        if !local_only {}

        responder.resolve();
    }

    fn disable_discovery(&mut self, responder: DwebDisableDiscoveryResponder) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "disable discovery",
        ) {
            return;
        }

        if let Some(mdns) = &mut self.state.lock().mdns {
            if mdns.stop().is_err() {
                responder.reject();
            }
        }

        responder.resolve();
    }

    fn known_peers(&mut self, responder: DwebKnownPeersResponder) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "known peers") {
            return;
        }

        let peers: Vec<Peer> = {
            self.state
                .lock()
                .known_peers
                .values()
                .map(|item| item.peer.clone())
                .collect()
        };
        responder.resolve(Some(peers));
    }

    fn set_p2p_provider(&mut self, responder: DwebSetP2pProviderResponder, provider: ObjectRef) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "set p2p provider",
        ) {
            return;
        }

        // Check that we don't already have a UI provider setup.
        let mut state = self.state.lock();
        if state.p2p_provider.is_some() {
            error!(
                "Trying to set a duplicate p2p provider from {}",
                self.origin_attributes.identity()
            );
            responder.reject();
        }

        // Register the new provider.
        if let Some(DwebServiceProxy::P2pProvider(p2p_proxy)) = self.proxy_tracker.get(&provider) {
            state.p2p_provider = Some(p2p_proxy.clone());
        } else {
            responder.reject();
        }

        self.provides_p2p = true;
        responder.resolve();
    }

    fn pair_with(&mut self, responder: DwebPairWithResponder, peer: Peer) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "pair with") {
            return;
        }

        {
            let state = self.state.lock();
            // Check if this peer is in the current list of known peers.
            if !state.known_peers.contains_key(&peer.device_id) {
                responder.reject(ConnectError {
                    kind: ConnectErrorKind::NotConnected,
                    detail: "".into(),
                });
                return;
            }

            let peer = state.known_peers.get(&peer.device_id).unwrap();

            if !peer.is_local {
                error!("Remote peers are not supported yet!");
                responder.reject(ConnectError {
                    kind: ConnectErrorKind::NotConnected,
                    detail: "Remote Peers not supported yet!".into(),
                });
                return;
            }

            if let Some(ref mdns) = state.mdns {
                let endpoint = peer.endpoint;
                debug!("Will pair with {:?}", endpoint);
                let this_peer = mdns.get_peer();
                let remote_peer = peer.peer.clone();
                let state2 = self.state.clone();
                let _ = std::thread::Builder::new()
                    .name("mdns connect".into())
                    .spawn(move || {
                        let client = HandshakeClient::new(&endpoint);
                        match client.pair_with(this_peer) {
                            Ok(_) => {
                                let session = state2.lock().create_session(remote_peer);
                                responder.resolve(session);
                            }
                            Err(Status::Denied) => {
                                responder.reject(ConnectError {
                                    kind: ConnectErrorKind::Denied,
                                    detail: "ConnectError:Denied".into(),
                                });
                            }
                            Err(Status::NotConnected) => {
                                responder.reject(ConnectError {
                                    kind: ConnectErrorKind::NotConnected,
                                    detail: "ConnectError:NotConnected".into(),
                                });
                            }
                            Err(_) => {
                                responder.reject(ConnectError {
                                    kind: ConnectErrorKind::Other,
                                    detail: "ConnectError:Other".into(),
                                });
                            }
                        }
                    });
            } else {
                error!("No mdns available, can't connect!");
                responder.reject(ConnectError {
                    kind: ConnectErrorKind::Other,
                    detail: "No mDNS available".into(),
                });
            }
        }
    }

    fn dial(&mut self, responder: DwebDialResponder, session: Session, params: JsonValue) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "dweb",
            "setup webrtc for",
        ) {
            return;
        }

        let state = self.state.lock();

        // Find the peer for this session.
        let peer = if let Some(session) = state.sessions.get(&session.id) {
            session.peer.clone()
        } else {
            responder.reject(ConnectError {
                kind: ConnectErrorKind::NotPaired,
                detail: "No such session".into(),
            });
            return;
        };

        // Check if this peer is in the current list of known peers.
        if !state.known_peers.contains_key(&peer.device_id) {
            responder.reject(ConnectError {
                kind: ConnectErrorKind::NotConnected,
                detail: "".into(),
            });
            return;
        }

        let peer = state.known_peers.get(&peer.device_id).unwrap();

        if !peer.is_local {
            error!("Remote peers are not supported yet!");
            responder.reject(ConnectError {
                kind: ConnectErrorKind::NotConnected,
                detail: "Remote Peers not supported yet!".into(),
            });
            return;
        }

        if let Some(ref mdns) = state.mdns {
            let endpoint = peer.endpoint;
            info!("Will send params to {:?}", endpoint);
            let this_peer = mdns.get_peer();
            let _ = std::thread::Builder::new()
                .name("mdns connect".into())
                .spawn(move || {
                    let client = HandshakeClient::new(&endpoint);
                    match client.dial(this_peer, params) {
                        Ok(answer) => responder.resolve(answer),
                        Err(Status::Denied) => {
                            responder.reject(ConnectError {
                                kind: ConnectErrorKind::Denied,
                                detail: "".into(),
                            });
                        }
                        Err(Status::NotConnected) => {
                            responder.reject(ConnectError {
                                kind: ConnectErrorKind::NotConnected,
                                detail: "".into(),
                            });
                        }
                        Err(_) => {
                            responder.reject(ConnectError {
                                kind: ConnectErrorKind::Other,
                                detail: "".into(),
                            });
                        }
                    }
                });
        } else {
            error!("No mdns available, can't connect!");
            responder.reject(ConnectError {
                kind: ConnectErrorKind::Other,
                detail: "No mDNS available".into(),
            });
        }
    }

    fn get_session(&mut self, responder: DwebGetSessionResponder, id: String) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "get session") {
            return;
        }

        match self.state.lock().sessions.get(&id) {
            Some(session) => responder.resolve(session.clone()),
            None => responder.reject(SessionError::InvalidId),
        }
    }

    fn get_sessions(&mut self, responder: DwebGetSessionsResponder) {
        if responder.maybe_send_permission_error(&self.origin_attributes, "dweb", "get sessions") {
            return;
        }

        let sessions = self.state.lock().sessions.values().cloned().collect();
        responder.resolve(Some(sessions));
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
            provides_p2p: false,
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

        if self.provides_p2p {
            self.state.lock().p2p_provider = None;
        }

        // TODO: when needed
        // self.observers.clear(...);

        self.tracker.clear();
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
