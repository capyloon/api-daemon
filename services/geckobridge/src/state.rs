// Manages the state shared by GeckoBridge instances and exposes
// an api usable by other services.

use crate::generated::common::{
    AppsServiceDelegateProxy, CardInfoType, MobileManagerDelegateProxy, NetworkInfo,
    NetworkManagerDelegateProxy, NetworkOperator, PowerManagerDelegateProxy,
};
use common::tokens::SharedTokensManager;
use common::traits::{OriginAttributes, Shared};
use common::JsonValue;
use log::{debug, error};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DelegatorError {
    #[error("Report errors from web runtime")]
    InvalidWebRuntimeService,
    #[error("Receive receiver error")]
    InvalidChannel,
    #[error("Failed to get delegate manager")]
    InvalidDelegator,
}

pub enum PrefValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

lazy_static! {
    pub(crate) static ref GECKO_BRIDGE_SHARED_STATE: Shared<GeckoBridgeState> =
        Shared::adopt(GeckoBridgeState::default());
}

#[derive(Default)]
pub struct GeckoBridgeState {
    prefs: HashMap<String, PrefValue>,
    appsservice: Option<AppsServiceDelegateProxy>,
    powermanager: Option<PowerManagerDelegateProxy>,
    mobilemanager: Option<MobileManagerDelegateProxy>,
    networkmanager: Option<NetworkManagerDelegateProxy>,
    observers: Vec<Sender<()>>,
    tokens: SharedTokensManager,
}

impl GeckoBridgeState {
    /// Reset the state, making it possible to set new delegates.
    pub fn reset(&mut self) {
        self.prefs = HashMap::new();
        self.powermanager = None;
        self.appsservice = None;
        self.mobilemanager = None;
        self.networkmanager = None;
        // On session dropped, do no reset the observers.
    }

    /// Delegates that are common to device and desktop builds.
    pub fn common_delegates_ready(&self) -> bool {
        self.appsservice.is_some() && self.powermanager.is_some()
    }

    /// Delegates that are only available on device builds.
    pub fn device_delegates_ready(&self) -> bool {
        self.mobilemanager.is_some() && self.networkmanager.is_some()
    }

    /// true if all the expected delegates have been set.
    #[cfg(target_os = "android")]
    pub fn is_ready(&self) -> bool {
        self.common_delegates_ready() && self.device_delegates_ready()
    }

    /// true if all the expected delegates have been set.
    #[cfg(not(target_os = "android"))]
    pub fn is_ready(&self) -> bool {
        self.common_delegates_ready()
    }

    fn notify_readyness_observers(&mut self) {
        if !self.is_ready() {
            return;
        }
        for sender in &self.observers {
            let _ = sender.send(());
        }
    }

    // Return a 'Receiver' to receivce the update when all delegates are ready;
    pub fn observe_bridge(&mut self) -> Receiver<()> {
        let (sender, receiver) = channel();
        {
            self.observers.push(sender);
        }
        receiver
    }

    // Preferences related methods.
    pub fn set_bool_pref(&mut self, name: String, value: bool) {
        let _ = self.prefs.insert(name, PrefValue::Bool(value));
    }

    pub fn get_bool_pref(&self, name: &str) -> Option<bool> {
        match self.prefs.get(name) {
            Some(PrefValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn set_int_pref(&mut self, name: String, value: i64) {
        let _ = self.prefs.insert(name, PrefValue::Int(value));
    }

    pub fn get_int_pref(&self, name: &str) -> Option<i64> {
        match self.prefs.get(name) {
            Some(PrefValue::Int(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn set_char_pref(&mut self, name: String, value: String) {
        let _ = self.prefs.insert(name, PrefValue::Str(value));
    }

    pub fn get_char_pref(&self, name: &str) -> Option<String> {
        match self.prefs.get(name) {
            Some(PrefValue::Str(value)) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_pref(&self, name: &str) -> Option<PrefValue> {
        match self.prefs.get(name) {
            Some(PrefValue::Bool(value)) => Some(PrefValue::Bool(*value)),
            Some(PrefValue::Int(value)) => Some(PrefValue::Int(*value)),
            Some(PrefValue::Str(value)) => Some(PrefValue::Str(value.clone())),
            _ => None,
        }
    }

    // Power manager delegate management.
    pub fn set_powermanager_delegate(&mut self, delegate: PowerManagerDelegateProxy) {
        self.powermanager = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn powermanager_set_screen_enabled(&mut self, value: bool, is_external_screen: bool) {
        if let Some(powermanager) = &mut self.powermanager {
            let _ = powermanager.set_screen_enabled(value, is_external_screen);
        } else {
            error!("The powermanager delegate is not set!");
        }
    }

    // Apps service delegate management.
    pub fn is_apps_service_ready(&self) -> bool {
        self.appsservice.is_some()
    }

    pub fn set_apps_service_delegate(&mut self, delegate: AppsServiceDelegateProxy) {
        self.appsservice = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn apps_service_on_boot(&mut self, manifest_url: String, value: JsonValue) {
        debug!("apps_service_on_update: {} - {:?}", &manifest_url, value);
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_boot(manifest_url, value);
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    pub fn apps_service_on_install(&mut self, manifest_url: String, value: JsonValue) {
        debug!("apps_service_on_update: {} - {:?}", &manifest_url, value);
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_install(manifest_url, value);
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    pub fn apps_service_on_update(&mut self, manifest_url: String, value: JsonValue) {
        debug!("apps_service_on_update: {} - {:?}", &manifest_url, value);
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_update(manifest_url, value);
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    pub fn apps_service_on_uninstall(&mut self, manifest_url: String) {
        debug!("apps_service_on_uninstall: {}", &manifest_url);
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_uninstall(manifest_url);
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    // CardInfo manager delegate management.
    pub fn set_mobilemanager_delegate(&mut self, delegate: MobileManagerDelegateProxy) {
        self.mobilemanager = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn mobilemanager_get_cardinfo(
        &mut self,
        service_id: i64,
        info_type: CardInfoType,
    ) -> Result<String, DelegatorError> {
        if let Some(mobilemanager) = &mut self.mobilemanager {
            let rx = mobilemanager.get_card_info(service_id, info_type);
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(info) => Ok(info),
                    Err(_) => Err(DelegatorError::InvalidWebRuntimeService),
                }
            } else {
                Err(DelegatorError::InvalidChannel)
            }
        } else {
            Err(DelegatorError::InvalidDelegator)
        }
    }

    pub fn mobilemanager_get_mnc_mcc(
        &mut self,
        service_id: i64,
        is_sim: bool,
    ) -> Result<NetworkOperator, DelegatorError> {
        if let Some(mobilemanager) = &mut self.mobilemanager {
            let rx = mobilemanager.get_mnc_mcc(service_id, is_sim);
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(operator) => Ok(operator),
                    Err(_) => Err(DelegatorError::InvalidWebRuntimeService),
                }
            } else {
                Err(DelegatorError::InvalidChannel)
            }
        } else {
            Err(DelegatorError::InvalidDelegator)
        }
    }

    // Network manager delegate management.
    pub fn set_networkmanager_delegate(&mut self, delegate: NetworkManagerDelegateProxy) {
        self.networkmanager = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn networkmanager_get_network_info(&mut self) -> Result<NetworkInfo, DelegatorError> {
        if let Some(networkmanager) = &mut self.networkmanager {
            let rx = networkmanager.get_network_info();
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(info) => Ok(info),
                    Err(_) => Err(DelegatorError::InvalidWebRuntimeService),
                }
            } else {
                Err(DelegatorError::InvalidChannel)
            }
        } else {
            Err(DelegatorError::InvalidDelegator)
        }
    }

    pub fn register_token(&mut self, token: &str, origin_attribute: OriginAttributes) -> bool {
        self.tokens.lock().register(token, origin_attribute)
    }

    pub fn get_tokens_manager(&self) -> SharedTokensManager {
        self.tokens.clone()
    }
}
