// The Gecko Bridge service.

use super::state::*;
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, TrackerId,
};
use log::{error, info};
use std::collections::{HashMap, HashSet};

pub struct GeckoBridgeService {
    id: TrackerId,
    proxy_tracker: GeckoBridgeProxyTracker,
    state: Shared<GeckoBridgeState>,
}

impl GeckoBridge for GeckoBridgeService {
    fn get_proxy_tracker(&mut self) -> &mut GeckoBridgeProxyTracker {
        &mut self.proxy_tracker
    }
}

impl GeckoFeaturesMethods for GeckoBridgeService {
    fn bool_pref_changed(
        &mut self,
        responder: &GeckoFeaturesBoolPrefChangedResponder,
        pref_name: String,
        value: bool,
    ) {
        self.state.lock().set_bool_pref(pref_name, value);
        responder.resolve();
    }

    fn char_pref_changed(
        &mut self,
        responder: &GeckoFeaturesCharPrefChangedResponder,
        pref_name: String,
        value: String,
    ) {
        self.state.lock().set_char_pref(pref_name, value);
        responder.resolve();
    }

    fn int_pref_changed(
        &mut self,
        responder: &GeckoFeaturesIntPrefChangedResponder,
        pref_name: String,
        value: i64,
    ) {
        self.state.lock().set_int_pref(pref_name, value);
        responder.resolve();
    }

    fn set_power_manager_delegate(
        &mut self,
        responder: &GeckoFeaturesSetPowerManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        // Get the proxy and update our state.
        match self.proxy_tracker.get(&delegate) {
            Some(GeckoBridgeProxy::PowerManagerDelegate(delegate)) => {
                self.state
                    .lock()
                    .set_powermanager_delegate(delegate.clone());
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked powermanager delegate.");
                responder.reject();
            }
        }
    }

    fn set_apps_service_delegate(
        &mut self,
        responder: &GeckoFeaturesSetAppsServiceDelegateResponder,
        delegate: ObjectRef,
    ) {
        // Get the proxy and update our state.
        match self.proxy_tracker.get(&delegate) {
            Some(GeckoBridgeProxy::AppsServiceDelegate(delegate)) => {
                self.state
                    .lock()
                    .set_apps_service_delegate(delegate.clone());
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked apps service delegate");
                responder.reject();
            }
        }
    }

    fn set_mobile_manager_delegate(
        &mut self,
        responder: &GeckoFeaturesSetMobileManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        // Get the proxy and update our state.
        match self.proxy_tracker.get(&delegate) {
            Some(GeckoBridgeProxy::MobileManagerDelegate(delegate)) => {
                self.state
                    .lock()
                    .set_mobilemanager_delegate(delegate.clone());
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked mobilemanager delegate.");
                responder.reject();
            }
        }
    }

    fn set_network_manager_delegate(
        &mut self,
        responder: &GeckoFeaturesSetNetworkManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        // Get the proxy and update our state.
        match self.proxy_tracker.get(&delegate) {
            Some(GeckoBridgeProxy::NetworkManagerDelegate(delegate)) => {
                self.state
                    .lock()
                    .set_networkmanager_delegate(delegate.clone());
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked networkmanager delegate.");
                responder.reject();
            }
        }
    }

    fn register_token(
        &mut self,
        responder: &GeckoFeaturesRegisterTokenResponder,
        token: String,
        url: String,
        permissions: Option<Vec<String>>,
    ) {
        let permissions_set = match permissions {
            Some(permissions) => {
                // Turn the Vec<String> into a HashSet<String>
                let mut set = HashSet::new();
                for perm in permissions {
                    set.insert(perm);
                }
                set
            }
            None => HashSet::new(),
        };
        let origin_attributes = OriginAttributes::new(&url, permissions_set);
        let mut state = self.state.lock();
        if state.register_token(&token, origin_attributes) {
            responder.resolve()
        } else {
            responder.reject();
        }
    }
}

impl Service<GeckoBridgeService> for GeckoBridgeService {
    // Shared among instances.
    type State = GeckoBridgeState;

    fn shared_state() -> Shared<Self::State> {
        let a = &*GECKO_BRIDGE_SHARED_STATE;
        a.clone()
    }

    fn create(
        attrs: &OriginAttributes,
        _context: SharedSessionContext,
        state: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Option<GeckoBridgeService> {
        info!("GeckoBridgeService::create");

        // Only connections from the UDS socket are permitted.
        if attrs.identity() != "uds" {
            error!("Only Gecko can get an instance of the GeckoBridge service!");
            return None;
        }

        // We only allow a single instance of this service.
        if state.lock().is_ready() {
            error!("Creating several instances of the GeckoBridge service is forbidden!");
            return None;
        }

        let service_id = helper.session_tracker_id().service();
        Some(GeckoBridgeService {
            id: service_id,
            proxy_tracker: HashMap::new(),
            state,
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<GeckoBridgeFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("SettingsService request: {:?}", req),
            Err(err) => format!("Unable to format SettingsService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        self.proxy_tracker.remove(&object_id.into()).is_some()
    }
}

impl Drop for GeckoBridgeService {
    fn drop(&mut self) {
        info!("Dropping GeckoBridge Service #{}", self.id);
        self.state.lock().reset();
    }
}
