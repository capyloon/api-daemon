/// Implementation of the telephony service.
use crate::generated::common::*;
use crate::generated::service::*;
use crate::PowerManagerSupport;
use common::core::BaseMessage;
use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, StateLogger,
};
use log::{debug, error};

pub struct PowerManagerState {
    cpu_sleep_allowed: bool,
    screen_enabled: bool,
    ext_screen_enabled: bool,
    factory_reset: FactoryResetReason,
    key_enabled: bool,
    screen_brightness: i64,
    ext_screen_brightness: i64,
    key_light_brightness: i64,
    init_done: bool,
}

impl Default for PowerManagerState {
    fn default() -> Self {
        Self {
            cpu_sleep_allowed: true,
            screen_enabled: true,
            ext_screen_enabled: true,
            factory_reset: FactoryResetReason::Normal,
            key_enabled: true,
            screen_brightness: 100,
            ext_screen_brightness: 100,
            key_light_brightness: 100,
            init_done: false,
        }
    }
}

impl PowerManagerState {
    // Set the hardware in sync with the current state.
    fn init(&mut self, provider: &mut Box<dyn PowerManagerSupport>) {
        if self.init_done {
            error!("We should not initialize PowerManagerState more than once!");
            return;
        }

        // CPU wakelock
        provider.set_cpu_sleep_allowed(self.cpu_sleep_allowed);

        // Main screen
        provider.set_screen_state(self.screen_enabled, 0);
        provider.set_screen_brightness(self.screen_brightness as _, 0);

        // TODO, we should prevent control state of secondary screen if
        // the device doesn't have boot animation file to prevent showing
        // noise in external screen.

        // Key lights.
        provider.set_key_light_enabled(self.key_enabled);
        provider.set_key_light_brightness(self.key_light_brightness as _);

        self.init_done = true;
    }
}

impl StateLogger for PowerManagerState {}

pub struct PowerManager {
    shared_obj: Shared<PowerManagerState>,
    inner: Box<dyn PowerManagerSupport>,
}

impl PowerManager {
    fn screen_brightness(&mut self, value: i64, is_external: bool) -> bool {
        debug!(
            "screen_brightness: brightness {} is external {}",
            value, is_external
        );
        if !(0..=100).contains(&value) {
            error!("set_screen_brightness: invalid brightness {}", value);
            return false;
        }

        if is_external {
            self.shared_obj.lock().ext_screen_brightness = value;
            return self
                .inner
                .set_screen_brightness(value as _, 1 /* external screen index */);
        }

        self.shared_obj.lock().screen_brightness = value;
        self.inner
            .set_screen_brightness(value as _, 0 /* main screen index */)
    }

    fn screen_enabled(&mut self, state: bool, is_external: bool) -> bool {
        debug!(
            "screen_enabled: state {}, is external {}",
            state, is_external
        );

        if is_external {
            self.shared_obj.lock().ext_screen_enabled = state;
        } else {
            self.shared_obj.lock().screen_enabled = state;
        }

        self.inner
            .set_screen_state(state, if is_external { 1 } else { 0 })
    }
}

impl PowermanagerService for PowerManager {}

impl PowermanagerMethods for PowerManager {
    fn control_screen(
        &mut self,
        responder: &PowermanagerControlScreenResponder,
        info: crate::generated::common::ScreenControlInfo,
    ) {
        debug!("Control_screen {:?} ", info);

        // Hanlde the screen enable/disable and backlight with
        // different order to prevent abnormal display.
        if info.state.is_some() && info.brightness.is_some() {
            let state = match info.state.unwrap() {
                ScreenState::On => true,
                ScreenState::Off => false,
            };

            let brightness = info.brightness.expect("Invalid brightness");
            // Enabled the screen before turn on the backlight.
            if state {
                if !self.screen_enabled(state, info.is_external) {
                    responder.reject();
                    return;
                }
                if !self.screen_brightness(brightness, info.is_external) {
                    responder.reject();
                    return;
                }
                responder.resolve();
                return;
            }
            // Turn off the backlight before disable the screen.
            if !self.screen_brightness(brightness, info.is_external) {
                responder.reject();
                return;
            }
            if !self.screen_enabled(state, info.is_external) {
                responder.reject();
                return;
            }
            responder.resolve();
            return;
        }

        if let Some(screen_state) = info.state {
            let state = match screen_state {
                ScreenState::On => true,
                ScreenState::Off => false,
            };

            if !self.screen_enabled(state, info.is_external) {
                responder.reject();
                return;
            }
        }

        if let Some(brightness) = info.brightness {
            if !self.screen_brightness(brightness, info.is_external) {
                responder.reject();
                return;
            }
        }
        responder.resolve();
    }

    fn power_off(&mut self, responder: &PowermanagerPowerOffResponder) {
        debug!("power_off");
        responder.resolve();
        self.inner.power_off();
    }

    fn reboot(&mut self, responder: &PowermanagerRebootResponder) {
        debug!("reboot");
        responder.resolve();
        self.inner.reboot();
    }

    fn get_cpu_sleep_allowed(&mut self, responder: &PowermanagerGetCpuSleepAllowedResponder) {
        debug!("get_cpu_sleep_allowed");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.cpu_sleep_allowed);
    }

    fn set_cpu_sleep_allowed(&mut self, value: bool) {
        debug!("set_cpu_sleep_allowed");
        {
            let mut shared = self.shared_obj.lock();
            shared.cpu_sleep_allowed = value;
        }

        self.inner.set_cpu_sleep_allowed(value);
    }

    fn get_ext_screen_brightness(
        &mut self,
        responder: &PowermanagerGetExtScreenBrightnessResponder,
    ) {
        let shared = self.shared_obj.lock();
        responder.resolve(shared.ext_screen_brightness);
        debug!("get_ext_screen_brightness {}", shared.ext_screen_brightness);
    }

    fn set_ext_screen_brightness(&mut self, value: i64) {
        if !self.screen_brightness(value, true) {
            error!("Failed to set external screen brightness");
        }
    }

    fn get_ext_screen_enabled(&mut self, responder: &PowermanagerGetExtScreenEnabledResponder) {
        let shared = self.shared_obj.lock();
        responder.resolve(shared.ext_screen_enabled);
        debug!("get_ext_screen_enabled {}", shared.ext_screen_enabled);
    }

    fn set_ext_screen_enabled(&mut self, value: bool) {
        if !self.screen_enabled(value, true) {
            error!("Failed to set external screen");
        }
    }

    fn get_factory_reset(&mut self, responder: &PowermanagerGetFactoryResetResponder) {
        debug!("get_factory_reset");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.factory_reset);
    }

    fn set_factory_reset(&mut self, value: FactoryResetReason) {
        debug!("set_factory_reset");
        {
            let mut shared = self.shared_obj.lock();
            shared.factory_reset = value;
        }
        self.inner.set_factory_reset_reason(value);
    }

    fn get_key_light_brightness(&mut self, responder: &PowermanagerGetKeyLightBrightnessResponder) {
        debug!("get_key_light_brightness");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.key_light_brightness);
    }

    fn set_key_light_brightness(&mut self, value: i64) {
        self.shared_obj.lock().key_light_brightness = value;
        let clamped = if value < 0 {
            0
        } else if value > 100 {
            100
        } else {
            value
        };
        self.inner.set_key_light_brightness(clamped as _);
    }

    fn get_key_light_enabled(&mut self, responder: &PowermanagerGetKeyLightEnabledResponder) {
        debug!("get_key_light_enabled");
        responder.resolve(self.shared_obj.lock().key_enabled);
    }

    fn set_key_light_enabled(&mut self, value: bool) {
        self.shared_obj.lock().key_enabled = value;
        self.inner.set_key_light_enabled(value);
    }

    fn get_screen_brightness(&mut self, responder: &PowermanagerGetScreenBrightnessResponder) {
        debug!("get_screen_brightness");
        responder.resolve(self.shared_obj.lock().screen_brightness);
    }

    fn set_screen_brightness(&mut self, value: i64) {
        if !self.screen_brightness(value, false) {
            error!("Failed to set screen brightness");
        }
    }

    fn get_screen_enabled(&mut self, responder: &PowermanagerGetScreenEnabledResponder) {
        debug!("get_screen_enabled");
        responder.resolve(self.shared_obj.lock().screen_enabled);
    }

    fn set_screen_enabled(&mut self, value: bool) {
        if !self.screen_enabled(value, false) {
            error!("Failed to set screen");
        }
    }
}

impl Service<PowerManager> for PowerManager {
    // Shared among instances.
    type State = PowerManagerState;

    fn shared_state() -> Shared<Self::State> {
        Shared::adopt(PowerManagerState::default())
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        shared_obj: Shared<Self::State>,
        _helper: SessionSupport,
    ) -> Result<PowerManager, String> {
        debug!("PowerManager::create");
        let mut inner = crate::platform::get_platform_support();
        shared_obj.lock().init(&mut inner);

        let service = PowerManager { shared_obj, inner };

        Ok(service)
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<PowermanagerServiceFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("PowerManagerService request: {:?}", req),
            Err(err) => format!("Unable to format PowerManagerService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        true
    }
}

impl Drop for PowerManager {
    fn drop(&mut self) {
        debug!("Dropping Powermanager Service");
    }
}
