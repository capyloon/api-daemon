// The Gecko Bridge service.

use super::state::*;
use crate::generated::common::*;
use crate::generated::{self, service::*};
use common::core::BaseMessage;
use common::traits::{
    EmptyConfig, OriginAttributes, Service, SessionSupport, Shared, SharedServiceState,
    SharedSessionContext, TrackerId,
};
use contacts_service::generated::common::SimContactInfo;
use contacts_service::service::ContactsService;
use log::{error, info};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use threadpool::ThreadPool;

lazy_static! {
    pub(crate) static ref PROXY_TRACKER: Arc<Mutex<GeckoBridgeProxyTracker>> =
        Arc::new(Mutex::new(GeckoBridgeProxyTracker::default()));
}

macro_rules! getDelegateWrapper {
    ($struct_name: ident, $fn_name: ident, $delegate: ident, $ret_type: ty) => {
        impl $struct_name {
            fn $fn_name(&mut self, delegate: ObjectRef) -> Option<$ret_type> {
                match self.get_proxy_tracker().lock().get(&delegate) {
                    Some(GeckoBridgeProxy::$delegate(delegate)) => Some(delegate.clone()),
                    _ => None,
                }
            }
        }
    };
}

pub struct GeckoBridgeService {
    id: TrackerId,
    state: Shared<GeckoBridgeState>,
    only_register_token: bool,
    pool: ThreadPool,
}

getDelegateWrapper!(
    GeckoBridgeService,
    get_app_service_delegate,
    AppsServiceDelegate,
    AppsServiceDelegateProxy
);
getDelegateWrapper!(
    GeckoBridgeService,
    get_power_manager_delegate,
    PowerManagerDelegate,
    PowerManagerDelegateProxy
);
getDelegateWrapper!(
    GeckoBridgeService,
    get_mobile_manager_delegate,
    MobileManagerDelegate,
    MobileManagerDelegateProxy
);
getDelegateWrapper!(
    GeckoBridgeService,
    get_network_manager_delegate,
    NetworkManagerDelegate,
    NetworkManagerDelegateProxy
);
getDelegateWrapper!(
    GeckoBridgeService,
    get_preference_delegate,
    PreferenceDelegate,
    PreferenceDelegateProxy
);

impl GeckoBridge for GeckoBridgeService {
    fn get_proxy_tracker(&mut self) -> Arc<Mutex<GeckoBridgeProxyTracker>> {
        let a = &*PROXY_TRACKER;
        a.clone()
    }
}

impl GeckoFeaturesMethods for GeckoBridgeService {
    fn bool_pref_changed(
        &mut self,
        responder: GeckoFeaturesBoolPrefChangedResponder,
        pref_name: &str,
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
        responder: GeckoFeaturesCharPrefChangedResponder,
        pref_name: &str,
        value: &str,
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
        responder: GeckoFeaturesIntPrefChangedResponder,
        pref_name: &str,
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
        responder: GeckoFeaturesSetPowerManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        // Get the proxy and update our state.
        if let Some(power_delegate) = self.get_power_manager_delegate(delegate) {
            self.state.lock().set_powermanager_delegate(power_delegate);
            responder.resolve();
        } else {
            responder.reject();
        }
    }

    fn set_apps_service_delegate(
        &mut self,
        responder: GeckoFeaturesSetAppsServiceDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        // Get the proxy and update our state.
        if let Some(app_delegate) = self.get_app_service_delegate(delegate) {
            self.state.lock().set_apps_service_delegate(app_delegate);
            responder.resolve();
        } else {
            responder.reject();
        }
    }

    fn set_mobile_manager_delegate(
        &mut self,
        responder: GeckoFeaturesSetMobileManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        // Get the proxy and update our state.
        if let Some(mobile_delegate) = self.get_mobile_manager_delegate(delegate) {
            self.state
                .lock()
                .set_mobilemanager_delegate(mobile_delegate);
            responder.resolve();
        } else {
            responder.reject();
        }
    }

    fn set_network_manager_delegate(
        &mut self,
        responder: GeckoFeaturesSetNetworkManagerDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        // Get the proxy and update our state.
        if let Some(network_delegate) = self.get_network_manager_delegate(delegate) {
            self.state
                .lock()
                .set_networkmanager_delegate(network_delegate);
            responder.resolve();
        } else {
            responder.reject();
        }
    }

    fn set_preference_delegate(
        &mut self,
        responder: GeckoFeaturesSetPreferenceDelegateResponder,
        delegate: ObjectRef,
    ) {
        if self.only_register_token {
            responder.reject();
            return;
        }

        // Get the proxy and update our state.
        if let Some(pref_delegate) = self.get_preference_delegate(delegate) {
            self.state.lock().set_preference_delegate(pref_delegate);
            responder.resolve();
        } else {
            responder.reject();
        }
    }

    fn register_token(
        &mut self,
        responder: GeckoFeaturesRegisterTokenResponder,
        token: &str,
        url: &str,
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
        let origin_attributes = OriginAttributes::new(url, permissions_set);
        let shared_state = self.state.clone();
        let token = token.to_owned();
        self.pool.execute(move || {
            if shared_state
                .lock()
                .register_token(&token, origin_attributes)
            {
                responder.resolve();
            } else {
                responder.reject();
            }
        });
    }

    fn import_sim_contacts(
        &mut self,
        responder: GeckoFeaturesImportSimContactsResponder,
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
                        category: x.category.to_string(),
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

common::impl_shared_state!(GeckoBridgeService, GeckoBridgeState, EmptyConfig);

impl Service<GeckoBridgeService> for GeckoBridgeService {
    fn create(
        attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<GeckoBridgeService, String> {
        info!("GeckoBridgeService::create");

        // Only connections from the UDS socket are permitted.
        if attrs.identity() != "uds" {
            error!("Only Gecko can get an instance of the GeckoBridge service!");
            return Err("Non Gecko client".into());
        }

        // We only allow a single instance of this service to change delegates.
        // Content processes that will connect afterwards can still use it
        // to register tokens.
        let state = Self::shared_state();
        let only_register_token = state.lock().is_ready();
        let service_id = helper.session_tracker_id().service();
        let pool = state.lock().pool.clone();
        Ok(GeckoBridgeService {
            id: service_id,
            state,
            only_register_token,
            pool,
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
        self.get_proxy_tracker()
            .lock()
            .remove(&object_id.into())
            .is_some()
    }
}

impl Drop for GeckoBridgeService {
    fn drop(&mut self) {
        info!("Dropping GeckoBridge Service #{}", self.id);

        // Reset the bridge state only if the instance exposing the
        // delegates is dropped.
        if !self.only_register_token {
            self.state.lock().reset();
        }
    }
}
