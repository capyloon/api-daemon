pub mod generated;
pub mod platform;
pub mod service;

use log::debug;
use std::io;

// Trait that needs to be implemented by platform-specific implementations.
// All functions have a default no-op implementation, allowing for incomplete
// platform implementations.
pub(crate) trait TimeManagerSupport {
    /// Change the timezone according to the argument, eg. America/Los_Angeles
    fn set_timezone(timezone: &str) -> Result<bool, io::Error> {
        debug!("TimeManagerSupport::set_timezone {}", timezone);
        Ok(false)
    }

    /// Returns the string represenation of the current timezone.
    fn get_timezone() -> Result<String, io::Error> {
        Ok("UTC-00:00".to_owned())
    }

    /// Sets the system clock to the given milliseconds since EPOCH.
    fn set_system_clock(msec: i64) -> Result<bool, io::Error> {
        debug!("TimeManagerSupport::set_system_clock {}", msec);

        Ok(true)
    }

    /// Returns the system clock in milliseconds since EPOCH.
    fn get_system_clock() -> Result<i64, io::Error> {
        debug!("TimeManagerSupport::get_system_clock");
        Ok(0)
    }

    /// Returns device runtime in milliseconds since boot.
    fn get_elapsed_real_time() -> Result<i64, io::Error> {
        debug!("TimeManagerSupport::get_elapsed_real_time");
        Ok(0)
    }
}
