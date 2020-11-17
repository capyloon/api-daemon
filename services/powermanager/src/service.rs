/// Implementation of the telephony service.
use crate::generated::common::*;
use crate::generated::service::*;
use android_utils::AndroidProperties;
use common::core::BaseMessage;
use common::traits::{OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext};
use geckobridge::service::*;
use log::{debug, error};
use recovery::AndroidRecovery;
use std::{thread, time};

const ANDROID_RB_RESTART: u32 = 0xDEAD_0001;
const ANDROID_RB_POWEROFF: u32 = 0xDEAD_0002;
const ANDROID_RB_RESTART2: u32 = 0xDEAD_0003;
const ANDROID_RB_THERMOFF: u32 = 0xDEAD_0004;

// Flash mode
const FLASH_NONE: i32 = 0;
// const FLASH_TIMED: i32 = 1;
// const FLASH_HARDWARE: i32 = 2;

const BRIGHTNESS_USER: i32 = 0;
// const BRIGHTNESS_SENSOR: i32 = 1;
// const BRIGHTNESS_LOW_PERSISTENCE: i32 = 2;

const TYPE_BACKLIGHT: i32 = 0;
const TYPE_KEYBOARD: i32 = 1;
const TYPE_BUTTONS: i32 = 2;
// const TYPE_BATTERY: i32 = 3;
// const TYPE_NOTIFICATIONS: i32 = 4;
// const TYPE_ATTENTION: i32 = 5;
// const TYPE_BLUETOOTH: i32 = 6;
// const TYPE_WIFI: i32 = 7;

const SERVICE: &str = "default";

pub struct SharedObj {
    cpu_allowed: bool,
    screen_enabled: bool,
    ext_screen_enabled: bool,
    factory_reset: FactoryResetReason,
    key_enabled: bool,
    screen_brightness: i64,
    ext_screen_brightness: i64,
    key_light_brightness: i64,
}

pub struct PowerManager {
    shared_obj: Shared<SharedObj>,
    light_service: Option<light::ILight>,
}

impl PowerManager {
    fn power_ctl(&mut self, reason: &str, cmd: u32) {
        debug!("Receive powerOff request {} ", reason);

        // This invokes init's power_ctl builtin via /init.rc.
        if let Err(err) = android_utils::AndroidProperties::set("sys.powerctl", reason) {
            error!("Failed to set sys.powerctl to '{}' : {:?}", reason, err);
        }

        // Device should reboot in few moments, but if it doesn't - call
        // android_reboot() to make sure that init isn't stuck somewhere
        let ten_secs = time::Duration::from_secs(10);
        thread::sleep(ten_secs);

        let restart_cmd = match cmd {
            ANDROID_RB_RESTART | ANDROID_RB_RESTART2 => "reboot",
            ANDROID_RB_POWEROFF => "shutdown",
            ANDROID_RB_THERMOFF => "thermal-shutdown",
            _ => {
                error!("Invalid power command");
                "invalid"
            }
        };
        debug!("reason = {} command = {}", reason, restart_cmd);

        if let Err(err) = AndroidProperties::set("sys.powerctl", restart_cmd) {
            error!(
                "Failed to set sys.powerctl to '{}' : {:?}",
                restart_cmd, err
            );
        }
    }

    fn ensure_service(&mut self) -> bool {
        if self.light_service.is_some() {
            true
        } else {
            self.light_service = light::ILight::get_service(SERVICE);
            if self.light_service.is_some() {
                true
            } else {
                error!("Failed to get service {}", SERVICE);
                false
            }
        }
    }
}

impl PowermanagerService for PowerManager {}

impl PowermanagerMethods for PowerManager {
    fn power_off(&mut self, responder: &PowermanagerPowerOffResponder) {
        debug!("power_off");
        responder.resolve();
        self.power_ctl("shutdown", ANDROID_RB_POWEROFF);
    }

    fn reboot(&mut self, responder: &PowermanagerRebootResponder) {
        debug!("reboot");
        responder.resolve();
        self.power_ctl("reboot", ANDROID_RB_RESTART);
    }

    fn get_cpu_sleep_allowed(&mut self, responder: &PowermanagerGetCpuSleepAllowedResponder) {
        debug!("get_cpu_sleep_allowed");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.cpu_allowed);
    }

    fn set_cpu_sleep_allowed(&mut self, value: bool) {
        debug!("set_cpu_sleep_allowed");
        {
            let mut shared = self.shared_obj.lock();
            shared.cpu_allowed = value;
        }
        android_utils::update_cpu_sleep_state(value);
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
        let brightness = ((value as f32 * 255.0) / (100.0)).round() as u32;
        debug!("set_ext_screen_brightness {} {}", value, brightness);
        let _ = android_utils::set_ext_screen_brightness(brightness);
        let mut shared = self.shared_obj.lock();
        shared.ext_screen_brightness = value;
    }

    fn get_ext_screen_enabled(&mut self, responder: &PowermanagerGetExtScreenEnabledResponder) {
        let shared = self.shared_obj.lock();
        responder.resolve(shared.ext_screen_enabled);
        debug!("get_ext_screen_enabled {}", shared.ext_screen_enabled);
    }

    fn set_ext_screen_enabled(&mut self, value: bool) {
        debug!("set_ext_screen_enabled {}", value);
        {
            let mut shared = self.shared_obj.lock();
            shared.ext_screen_enabled = value;
        }
        // Relay the request to Gecko using the bridge.
        let bridge = GeckoBridgeService::shared_state();
        bridge.lock().powermanager_set_screen_enabled(value, true);
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
        let res = AndroidRecovery::factory_reset(value as i32);
        if res < 0 {
            error!("Failed to do factory reset {}", res);
        }
    }

    fn get_key_light_brightness(&mut self, responder: &PowermanagerGetKeyLightBrightnessResponder) {
        debug!("get_key_light_brightness");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.key_light_brightness);
    }

    fn set_key_light_brightness(&mut self, value: i64) {
        {
            let mut shared = self.shared_obj.lock();
            shared.key_light_brightness = value;
        }

        let val = ((value as f32 * 255.0) / (100.0)).round() as u32;
        let color = 0xff00_0000 + (val << 16) + (val << 8) + val;

        let light_state = light::LightState {
            color,
            flash_mode: FLASH_NONE,
            flash_on_ms: 0,
            flash_off_ms: 0,
            brightness_mode: BRIGHTNESS_USER,
        };

        if self.ensure_service() {
            let s = &self.light_service.as_ref().unwrap();
            let _ = s.set_light(TYPE_BUTTONS, &light_state);
            let _ = s.set_light(TYPE_KEYBOARD, &light_state);
        }
    }

    fn get_key_light_enabled(&mut self, responder: &PowermanagerGetKeyLightEnabledResponder) {
        debug!("get_key_light_enabled");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.key_enabled);
    }

    fn set_key_light_enabled(&mut self, value: bool) {
        {
            let mut shared = self.shared_obj.lock();
            shared.key_enabled = value;
        }

        let color = if value { 0xffffffff } else { 0 };

        let light_state = light::LightState {
            color,
            flash_mode: FLASH_NONE,
            flash_on_ms: 0,
            flash_off_ms: 0,
            brightness_mode: BRIGHTNESS_USER,
        };

        if self.ensure_service() {
            let s = &self.light_service.as_ref().unwrap();
            let _ = s.set_light(TYPE_BUTTONS, &light_state);
            let _ = s.set_light(TYPE_KEYBOARD, &light_state);
        }
    }

    fn get_screen_brightness(&mut self, responder: &PowermanagerGetScreenBrightnessResponder) {
        debug!("get_screen_brightness");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.screen_brightness);
    }

    fn set_screen_brightness(&mut self, value: i64) {
        if !(0 <= value && value <= 100) {
            error!("set_screen_brightness: invalid request {}", value);
            return;
        }

        {
            let mut shared = self.shared_obj.lock();
            shared.screen_brightness = value;
        }

        let val = ((value as f32 * 255.0) / (100.0)).round() as u32;
        let color = 0xff00_0000 + (val << 16) + (val << 8) + val;

        let light_state = light::LightState {
            color,
            flash_mode: FLASH_NONE,
            flash_on_ms: 0,
            flash_off_ms: 0,
            brightness_mode: BRIGHTNESS_USER,
        };

        if self.ensure_service() {
            let s = self.light_service.as_ref().unwrap();
            let _ = s.set_light(TYPE_BACKLIGHT, &light_state);
        }
    }

    fn get_screen_enabled(&mut self, responder: &PowermanagerGetScreenEnabledResponder) {
        debug!("get_screen_enabled");
        let shared = self.shared_obj.lock();
        responder.resolve(shared.screen_enabled);
    }

    fn set_screen_enabled(&mut self, value: bool) {
        debug!("set_screen_enabled");
        {
            let mut shared = self.shared_obj.lock();
            shared.screen_enabled = value;
        }

        // Relay the request to Gecko using the bridge.
        let bridge = GeckoBridgeService::shared_state();
        bridge.lock().powermanager_set_screen_enabled(value, false);
    }
}

impl Service<PowerManager> for PowerManager {
    // Shared among instances.
    type State = SharedObj;

    fn shared_state() -> Shared<Self::State> {
        Shared::adopt(SharedObj {
            cpu_allowed: true,
            screen_enabled: true,
            ext_screen_enabled: true,
            factory_reset: FactoryResetReason::Normal,
            key_enabled: true,
            screen_brightness: 100,
            ext_screen_brightness: 100,
            key_light_brightness: 100,
        })
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        shared_obj: Shared<Self::State>,
        _helper: SessionSupport,
    ) -> Result<PowerManager, String> {
        debug!("PowerManager::create");
        let service = PowerManager {
            shared_obj,
            light_service: light::ILight::get_service(SERVICE),
        };

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
