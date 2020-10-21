// The Gecko Bridge service.

use super::state::*;
use crate::generated::common::*;
use crate::generated::{self, service::*};
use common::core::BaseMessage;
use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, TrackerId,
};
use log::{error, info};
use std::collections::{HashMap, HashSet};

use contacts_service::generated::common::SimContactInfo;
use contacts_service::service::ContactsService;

pub struct GeckoBridgeService {
    id: TrackerId,
    proxy_tracker: GeckoBridgeProxyTracker,
    state: Shared<GeckoBridgeState>,
    only_register_token: bool,
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
        if self.only_register_token {
            responder.reject();
            return;
        }
        self.state.lock().set_bool_pref(pref_name, value);
        responder.resolve();
    }

    fn char_pref_changed(
        &mut self,
        responder: &GeckoFeaturesCharPrefChangedResponder,
        pref_name: String,
        value: String,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }
        self.state.lock().set_char_pref(pref_name, value);
        responder.resolve();
    }

    fn int_pref_changed(
        &mut self,
        responder: &GeckoFeaturesIntPrefChangedResponder,
        pref_name: String,
        value: i64,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }
        self.state.lock().set_int_pref(pref_name, value);
        responder.resolve();
    }

    fn set_power_manager_delegate(
        &mut self,
        responder: &GeckoFeaturesSetPowerManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }
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
        if self.only_register_token {
            responder.reject();
            return;
        }
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
        if self.only_register_token {
            responder.reject();
            return;
        }
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
        if self.only_register_token {
            responder.reject();
            return;
        }
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

    fn import_sim_contacts(
        &mut self,
        responder: &GeckoFeaturesImportSimContactsResponder,
        sim_contacts: Option<Vec<generated::common::SimContactInfo>>,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        let sim_contact_info = match sim_contacts {
            Some(sim_contacts) => {
                let sim_contact_info = sim_contacts
                    .iter()
                    .map(|x| SimContactInfo {
                        id: x.id.to_string(),
                        tel: x.tel.to_string(),
                        email: x.email.to_string(),
                        name: x.name.to_string(),
                    })
                    .collect();
                sim_contact_info
            }
            None => Vec::new(),
        };

        let contact_state = ContactsService::shared_state();
        let mut contacts = contact_state.lock();
        match contacts.db.import_sim_contacts(&sim_contact_info) {
            Ok(()) => responder.resolve(),
            Err(err) => {
                error!("import_sim_contact error is {}", err);
                responder.reject()
            }
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

        // We only allow a single instance of this service to change delegates.
        // Content processes that will connect afterwards can still use it
        // to register tokens.
        let only_register_token = state.lock().is_ready();

        let service_id = helper.session_tracker_id().service();
        Some(GeckoBridgeService {
            id: service_id,
            proxy_tracker: HashMap::new(),
            state,
            only_register_token,
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
