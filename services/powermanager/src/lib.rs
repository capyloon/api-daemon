pub mod generated;
#[macro_use]
pub mod service;
pub mod platform;

use crate::generated::common::FactoryResetReason;
use log::debug;

// Trait that needs to be implemented by platform-specific implementations.
// All functions have a default no-op implementation, allowing for incomplete
// platform implementations.
pub trait PowerManagerSupport {
    /// Changes the screen brightness.
    /// value is the 0..100 percentage
    /// screen_id is the screen "number", 0-indexed.
    /// Returns whether the operation was successful.
    fn set_screen_brightness(&mut self, value: u8, screen_id: u8) -> bool {
        debug!(
            "PowerManagerSupport::set_screen_brightness to {} on screen {}",
            value, screen_id
        );
        true
    }

    /// Turns on or off the screen
    /// screen_id is the screen "number", 0-indexed.
    /// Returns whether the operation was successful.
    fn set_screen_state(&mut self, state: bool, screen_id: u8) -> bool {
        debug!(
            "PowerManagerSupport::set_screen_state to {} on screen {}",
            state, screen_id
        );
        true
    }

    /// Changes the key backlight brightness.
    /// value is the 0..100 percentage
    fn set_key_light_brightness(&mut self, value: u8) {
        debug!("PowerManagerSupport::set_key_light_brightness to {}", value);
    }

    /// Turns on or off the key backlight
    fn set_key_light_enabled(&mut self, value: bool) {
        debug!("PowerManagerSupport::set_key_light_enabled to {}", value);
    }

    /// Turns the device off.
    fn power_off(&mut self) {
        debug!("PowerManagerSupport::power_off");
    }

    /// Reboots the device.
    fn reboot(&mut self) {
        debug!("PowerManagerSupport::reboot");
    }

    /// Let the system know if the cpu can turn to sleep mode.
    fn set_cpu_sleep_allowed(&mut self, value: bool) {
        debug!("PowerManagerSupport::set_cpu_sleep_allowed to {}", value);
    }

    /// Sets the factory reset reason.
    fn set_factory_reset_reason(&mut self, reason: FactoryResetReason) {
        debug!(
            "PowerManagerSupport::set_factory_reset_reason to {}",
            reason as i32
        );
    }
}
