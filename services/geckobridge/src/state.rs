// Manages the state shared by GeckoBridge instances and exposes
// an api usable by other services.

use crate::generated::common::{
    AppsServiceDelegateProxy, CardInfoType, MobileManagerDelegateProxy, NetworkInfo,
    NetworkManagerDelegateProxy, NetworkOperator, ObjectRef, PowerManagerDelegateProxy,
    PreferenceDelegateProxy, WakelockProxy,
};
use crate::generated::service::{GeckoBridgeProxy, GeckoBridgeProxyTracker};
use crate::service::PROXY_TRACKER;
use common::tokens::SharedTokensManager;
use common::traits::{OriginAttributes, Shared, StateLogger};
use common::JsonValue;
use log::{debug, error};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub enum DelegateError {
    #[error("Report errors from web runtime")]
    InvalidWebRuntimeService,
    #[error("Receive receiver error")]
    InvalidChannel,
    #[error("Failed to get delegate manager")]
    InvalidDelegate,
    #[error("Invalid wakelock")]
    InvalidWakelock,
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
    preference: Option<PreferenceDelegateProxy>,
    mobilemanager: Option<MobileManagerDelegateProxy>,
    networkmanager: Option<NetworkManagerDelegateProxy>,
    observers: Vec<Sender<()>>,
    tokens: SharedTokensManager,
}

impl StateLogger for GeckoBridgeState {
    fn log(&self) {
        // We use the info log level to ensure this ends up in logcat even
        // when not in verbose log mode.
        use log::info;

        macro_rules! log_delegate {
            ($desc:expr,$name:ident) => {
                info!(
                    "  {:<25} [{}]",
                    format!("{} delegate:", $desc),
                    self.$name.is_some()
                );
            };
        }

        log_delegate!("Apps", appsservice);
        log_delegate!("Power Manager", powermanager);
        log_delegate!("Preferences", preference);
        log_delegate!("Mobile Manager", mobilemanager);
        log_delegate!("Network Manager", networkmanager);
    }
}

impl GeckoBridgeState {
    fn proxy_tracker(&mut self) -> Arc<Mutex<GeckoBridgeProxyTracker>> {
        let a = &*PROXY_TRACKER;
        a.clone()
    }

    /// Reset the state, making it possible to set new delegates.
    pub fn reset(&mut self) {
        self.prefs = HashMap::new();
        self.powermanager = None;
        self.preference = None;
        self.appsservice = None;
        self.mobilemanager = None;
        self.networkmanager = None;
        // Reset the proxy tracker content, which only holds proxy objects for the
        // delegates.
        let tracker = self.proxy_tracker();
        tracker.lock().clear();

        // On session dropped, do no reset the observers.
    }

    /// Delegates that are common to device and desktop builds.
    pub fn common_delegates_ready(&self) -> bool {
        self.appsservice.is_some() && self.powermanager.is_some() && self.preference.is_some()
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

    pub fn powermanager_set_screen_enabled(
        &mut self,
        value: bool,
        is_external_screen: bool,
    ) -> DelegateResponse<()> {
        match self.powermanager.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(powermanager) => DelegateResponse::from_receiver(
                powermanager.set_screen_enabled(value, is_external_screen),
            ),
        }
    }

    pub fn powermanager_request_wakelock(
        &mut self,
        topic: String,
    ) -> Result<ObjectRef, DelegateError> {
        if let Some(powermanager) = &mut self.powermanager {
            let rx = powermanager.request_wakelock(topic);
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(obj_ref) => {
                        if let Some(GeckoBridgeProxy::Wakelock(_proxy)) =
                            self.proxy_tracker().lock().get(&obj_ref)
                        {
                            debug!("Request the wakelock successfully.");
                            Ok(obj_ref)
                        } else {
                            error!("Failed to get wakelock: no proxy object.");
                            Err(DelegateError::InvalidWakelock)
                        }
                    }
                    Err(_) => {
                        error!("Failed to request wake lock, invalid object reference.");
                        Err(DelegateError::InvalidWakelock)
                    }
                }
            } else {
                error!("Failed to get the wakelock: invalid delegate channel.");
                Err(DelegateError::InvalidChannel)
            }
        } else {
            error!("Failed to get the wakelock: powermanager delegate is not set!");
            Err(DelegateError::InvalidDelegate)
        }
    }

    fn get_wakelock_proxy(&mut self, wakelock: ObjectRef) -> Result<WakelockProxy, DelegateError> {
        match self.proxy_tracker().lock().get(&wakelock) {
            Some(GeckoBridgeProxy::Wakelock(proxy)) => Ok(proxy.clone()),
            _ => Err(DelegateError::InvalidWakelock),
        }
    }

    pub fn powermanager_wakelock_get_topic(
        &mut self,
        wakelock: ObjectRef,
    ) -> Result<String, DelegateError> {
        if let Ok(mut proxy) = self.get_wakelock_proxy(wakelock) {
            let rx = proxy.get_topic();
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(topic) => {
                        debug!("powermanager_wakelock_get_topic: {}.", topic);
                        Ok(topic)
                    }
                    Err(_) => {
                        error!("powermanager_wakelock_get_topic: invalid wakelock.");
                        Err(DelegateError::InvalidWakelock)
                    }
                }
            } else {
                error!("powermanager_wakelock_get_topic: invalid channel.");
                Err(DelegateError::InvalidChannel)
            }
        } else {
            error!("powermanager_wakelock_get_topic: invalid wakelock proxy.");
            Err(DelegateError::InvalidWakelock)
        }
    }

    pub fn powermanager_wakelock_unlock(
        &mut self,
        wakelock: ObjectRef,
    ) -> Result<(), DelegateError> {
        let mut proxy = self.get_wakelock_proxy(wakelock)?;
        let rx = proxy.unlock();
        if let Ok(result) = rx.recv() {
            match result {
                Ok(()) => {
                    debug!("powermanager_wakelock_unlock: successful.");
                    Ok(())
                }
                Err(_) => {
                    error!("powermanager_wakelock_unlock: invalid channel.");
                    Err(DelegateError::InvalidChannel)
                }
            }
        } else {
            error!("powermanager_wakelock_unlock: invalid channel.");
            Err(DelegateError::InvalidChannel)
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

    pub fn apps_service_on_clear(
        &mut self,
        manifest_url: String,
        data_type: String,
        value: JsonValue,
    ) -> Result<(), DelegateError> {
        debug!("apps_service_on_clear: {}", &manifest_url);
        if let Some(service) = &mut self.appsservice {
            let rx = service.on_clear(manifest_url, data_type, value);
            if let Ok(result) = rx.recv() {
                match result {
                    Ok(_) => Ok(()),
                    Err(_) => Err(DelegateError::InvalidWebRuntimeService),
                }
            } else {
                error!("The apps service delegate rx channel error!");
                Err(DelegateError::InvalidChannel)
            }
        } else {
            error!("The apps service delegate is not set!");
            Err(DelegateError::InvalidDelegate)
        }
    }

    pub fn apps_service_on_boot(&mut self, manifest_url: String, value: JsonValue) {
        debug!("apps_service_on_boot: {} - {:?}", &manifest_url, value);
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_boot(manifest_url, value);
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    pub fn apps_service_on_boot_done(&mut self) {
        debug!("apps_service_on_boot_done");
        if let Some(service) = &mut self.appsservice {
            let _ = service.on_boot_done();
        } else {
            error!("The apps service delegate is not set!");
        }
    }

    pub fn apps_service_on_install(&mut self, manifest_url: String, value: JsonValue) {
        debug!("apps_service_on_install: {} - {:?}", &manifest_url, value);
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
    ) -> DelegateResponse<String> {
        match self.mobilemanager.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(mobilemanager) => {
                DelegateResponse::from_receiver(mobilemanager.get_card_info(service_id, info_type))
            }
        }
    }

    pub fn mobilemanager_get_mnc_mcc(
        &mut self,
        service_id: i64,
        is_sim: bool,
    ) -> DelegateResponse<NetworkOperator> {
        match self.mobilemanager.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(mobilemanager) => {
                DelegateResponse::from_receiver(mobilemanager.get_mnc_mcc(service_id, is_sim))
            }
        }
    }

    // Network manager delegate management.
    pub fn set_networkmanager_delegate(&mut self, delegate: NetworkManagerDelegateProxy) {
        self.networkmanager = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn networkmanager_get_network_info(&mut self) -> DelegateResponse<NetworkInfo> {
        match self.networkmanager.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(networkmanager) => {
                DelegateResponse::from_receiver(networkmanager.get_network_info())
            }
        }
    }

    // Preference delegate management.
    pub fn set_preference_delegate(&mut self, delegate: PreferenceDelegateProxy) {
        self.preference = Some(delegate);
        self.notify_readyness_observers();
    }

    pub fn preference_get_int(&mut self, pref_name: String) -> DelegateResponse<i64> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.get_int(pref_name)),
        }
    }

    pub fn preference_get_char(&mut self, pref_name: String) -> DelegateResponse<String> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.get_char(pref_name)),
        }
    }

    pub fn preference_get_bool(&mut self, pref_name: String) -> DelegateResponse<bool> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.get_bool(pref_name)),
        }
    }

    pub fn preference_set_int(&mut self, pref_name: String, value: i64) -> DelegateResponse<()> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.set_int(pref_name, value)),
        }
    }

    pub fn preference_set_char(
        &mut self,
        pref_name: String,
        value: String,
    ) -> DelegateResponse<()> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.set_char(pref_name, value)),
        }
    }

    pub fn preference_set_bool(&mut self, pref_name: String, value: bool) -> DelegateResponse<()> {
        match self.preference.as_mut() {
            None => DelegateResponse::from_error(DelegateError::InvalidDelegate),
            Some(prefs) => DelegateResponse::from_receiver(prefs.set_bool(pref_name, value)),
        }
    }

    pub fn register_token(&mut self, token: &str, origin_attribute: OriginAttributes) -> bool {
        self.tokens.lock().register(token, origin_attribute)
    }

    pub fn get_tokens_manager(&self) -> SharedTokensManager {
        self.tokens.clone()
    }
}

// A handle to receive the response from a delegate call without
// needing to hold the lock on the bridge shared state.
pub enum DelegateResponse<T> {
    Receiver(Receiver<Result<T, ()>>),
    Error(DelegateError),
}

impl<T> DelegateResponse<T> {
    pub fn from_receiver(receiver: Receiver<Result<T, ()>>) -> Self {
        DelegateResponse::Receiver(receiver)
    }

    pub fn from_error(error: DelegateError) -> Self {
        DelegateResponse::Error(error)
    }

    // Returns either the successfull result, or an error. If the enum had
    // been created with an error, just return it.
    pub fn get(&self) -> Result<T, DelegateError> {
        match &self {
            Self::Receiver(receiver) => receiver
                .recv()
                .map_err(|_| DelegateError::InvalidChannel)
                .and_then(|result| result.map_err(|_| DelegateError::InvalidWebRuntimeService)),
            Self::Error(err) => Err(err.clone()),
        }
    }
}
