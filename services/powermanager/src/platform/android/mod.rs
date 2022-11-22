/// Android specific implementation of the power manager trait.
use crate::generated::common::FactoryResetReason;
use crate::PowerManagerSupport;
use android_utils::AndroidProperties;
use common::traits::{Shared, SharedServiceState};
use geckobridge::service::*;
use geckobridge::state::GeckoBridgeState;
use light::{LightInterface, LightService, LightType};
use log::{debug, error};
use power::AndroidPower;
use recovery::AndroidRecovery;
use std::thread;
use std::time::Duration;

const ANDROID_RB_RESTART: u32 = 0xDEAD_0001;
const ANDROID_RB_POWEROFF: u32 = 0xDEAD_0002;
const ANDROID_RB_RESTART2: u32 = 0xDEAD_0003;
const ANDROID_RB_THERMOFF: u32 = 0xDEAD_0004;

// The cpu stays on, but the screen is off.
const PARTIAL_WAKE_LOCK: i32 = 1;
// The cpu and screen are on.
// const FULL_WAKE_LOCK: i32 = 2;

struct BridgeImpl {
    bridge: Shared<GeckoBridgeState>,
}

impl BridgeImpl {
    fn get_service() -> Self {
        debug!("ZZZ BridgeImpl::get_service start");
        Self {
            bridge: GeckoBridgeService::shared_state()
        }
    }
}

impl LightInterface for BridgeImpl {
    fn is_alive(&self) -> bool {
        self.bridge.lock().is_ready()
    }

    fn set_light_color(&self, light: LightType, _color: u32) -> Result<i32, ()> {
        Err(())
    }

    fn set_backlight_brightness(&self, brightness: f32) -> Result<i32, ()> {
        self.bridge.lock().powermanager_set_display_brightness(0, (brightness / 100.0) as _);
        Ok(0)
    }
}

#[derive(Default)]
pub struct AndroidPowerManager {
    light_service: Option<Box<dyn LightInterface>>,
}

impl AndroidPowerManager {
    fn power_ctl(&mut self, reason: &str, cmd: u32) {
        debug!("Receive powerOff request {} ", reason);

        // Set reboot reason.
        if let Err(err) = AndroidProperties::set("persist.b2g.reboot.reason", "normal") {
            error!(
                "Failed to set persist.b2g.reboot.reason to normal : {:?}",
                err
            );
        }

        // This invokes init's power_ctl builtin via /init.rc.
        if let Err(err) = AndroidProperties::set("sys.powerctl", reason) {
            error!("Failed to set sys.powerctl to '{}' : {:?}", reason, err);
        }

        // Device should reboot in few moments, but if it doesn't - call
        // android_reboot() to make sure that init isn't stuck somewhere
        let ten_secs = Duration::from_secs(10);
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

    fn is_alive(&self) -> bool {
        if let Some(light) = &self.light_service {
            return light.is_alive();
        }
        false
    }

    fn ensure_service(&mut self) -> bool {
        if self.is_alive() {
            return true;
        }

        let mut count = 0;
        let hundred_millis = Duration::from_millis(100);
        loop {
            count += 1;
            if let Some(service) = LightService::get_service() {
                self.light_service = Some(Box::new(service));
                return true;
            }
            error!("Failed to get light service, retry {}", count);
            if count > 5 {
                break;
            }
            thread::sleep(hundred_millis);
        }

        // Fallback to the bridged service if the bridge is ready.
        if GeckoBridgeService::shared_state().lock().is_ready() {
            self.light_service = Some(Box::new(BridgeImpl::get_service()));
            true
        } else {
            error!("GeckoBridge is not ready!");
            false
        }
    }
}

impl PowerManagerSupport for AndroidPowerManager {
    fn set_screen_brightness(&mut self, value: u8, screen_id: u8) -> bool {
        debug!(
            "AndroidPowerManager::set_screen_brightness to {} on screen {}",
            value, screen_id
        );

        // Set the backlight for external screen.
        if screen_id == 1 {
            let brightness = ((value as f32 * 255.0) / (100.0)).round() as u32;
            return android_utils::set_ext_screen_brightness(brightness);
        }

        // Set the backlight for main screen.
        if self.ensure_service() {
            let s = self.light_service.as_ref().expect("Invalid light service");
            if s.set_backlight_brightness(value as f32).is_ok() {
                return true;
            }
        }
        false
    }

    /// Turns on or off the screen
    /// screen_id is the screen "number", 0-indexed.
    fn set_screen_state(&mut self, state: bool, screen_id: u8) -> bool {
        debug!(
            "AndroidPowerManager::set_screen_state to {} on screen {}",
            state, screen_id
        );

        // Relay the request to Gecko using the bridge.
        let bridge = GeckoBridgeService::shared_state();
        let maybe_enabled = bridge
            .lock()
            .powermanager_set_screen_enabled(state, screen_id == 1);
        if maybe_enabled.get().is_err() {
            error!("Failed to set screen #{} to {}", screen_id, state);
            return false;
        }

        true
    }

    /// Changes the key backlight brightness.
    /// value is the 0..100 percentage
    fn set_key_light_brightness(&mut self, value: u8) {
        let val = ((value as f32 * 255.0) / (100.0)).round() as u32;
        let color = 0xff00_0000 + (val << 16) + (val << 8) + val;

        if self.ensure_service() {
            let s = &self.light_service.as_ref().unwrap();
            let _ = s.set_light_color(LightType::Buttons, color);
            let _ = s.set_light_color(LightType::Keyboard, color);
        }
    }

    /// Turns on or off the key backlight
    fn set_key_light_enabled(&mut self, value: bool) {
        let color = if value { 0xffffffff } else { 0 };

        if self.ensure_service() {
            let s = &self.light_service.as_ref().unwrap();
            let _ = s.set_light_color(LightType::Buttons, color);
            let _ = s.set_light_color(LightType::Keyboard, color);
        }
    }

    /// Turns the device off.
    fn power_off(&mut self) {
        self.power_ctl("shutdown", ANDROID_RB_POWEROFF);
    }

    /// Reboots the device.
    fn reboot(&mut self) {
        self.power_ctl("reboot", ANDROID_RB_RESTART);
    }

    /// Let the system know if the cpu can turn to sleep mode.
    fn set_cpu_sleep_allowed(&mut self, value: bool) {
        if value {
            AndroidPower::release_wake_lock("api-daemon").map_or_else(
                |err: String| error!("{}", err),
                |_| debug!("Release wake lock successfully"),
            );
        } else {
            AndroidPower::acquire_wake_lock(PARTIAL_WAKE_LOCK, "api-daemon").map_or_else(
                |err: String| error!("{}", err),
                |_| debug!("Acquire wake lock successfully"),
            );
        }
    }

    /// Sets the factory reset reason.
    fn set_factory_reset_reason(&mut self, reason: FactoryResetReason) {
        let res = AndroidRecovery::factory_reset(reason as i32);
        if res < 0 {
            error!("Failed to do factory reset {}", res);
        }
    }
}
