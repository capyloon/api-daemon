/// Linux implementationof TimeManagerSupport relying on b2ghald
use crate::TimeManagerSupport;
use b2ghald::client::SimpleClient;
use log::debug;
use std::io;

pub struct TimeManager {}

fn into_io_error() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "TimeManager")
}

impl TimeManagerSupport for TimeManager {
    /// Change the timezone according to the argument, eg. America/Los_Angeles
    fn set_timezone(timezone: &str) -> Result<bool, io::Error> {
        debug!("TimeManager::set_timezone {}", timezone);
        let mut hal = SimpleClient::new().ok_or_else(into_io_error)?;
        hal.set_timezone(timezone);
        Ok(true)
    }

    /// Returns the string represenation of the current timezone.
    fn get_timezone() -> Result<String, io::Error> {
        let mut hal = SimpleClient::new().ok_or_else(into_io_error)?;
        hal.get_timezone().ok_or_else(into_io_error)
    }

    /// Sets the system clock to the given milliseconds since EPOCH.
    fn set_system_clock(msec: i64) -> Result<bool, io::Error> {
        debug!("TimeManager::set_system_clock {}", msec);
        let mut hal = SimpleClient::new().ok_or_else(into_io_error)?;
        hal.set_system_time(msec);
        Ok(true)
    }

    /// Returns the system clock in milliseconds since EPOCH.
    fn get_system_clock() -> Result<i64, io::Error> {
        debug!("TimeManager::get_system_clock");
        let mut hal = SimpleClient::new().ok_or_else(into_io_error)?;
        Ok(hal.get_system_time())
    }

    /// Returns device runtime in milliseconds since boot.
    fn get_elapsed_real_time() -> Result<i64, io::Error> {
        debug!("TimeManager::get_elapsed_real_time");
        let mut hal = SimpleClient::new().ok_or_else(into_io_error)?;
        Ok(hal.get_uptime())
    }
}
