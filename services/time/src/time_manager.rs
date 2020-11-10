use libc::{gettimeofday, timeval};
use log::debug;
use std::io::Read;
use std::num::ParseFloatError;
use std::str::FromStr;
use std::{fs, io, ptr};

#[cfg(not(target_os = "android"))]
use std::env;

#[cfg(target_os = "android")]
use {
    crate::time_device::*,
    android_utils::AndroidProperties,
    libc::{settimeofday, suseconds_t, time_t},
    log::error,
    time::*,
};

#[derive(Clone, Debug)]
pub struct TimeManager {}

mod ffi {
    #[cfg(target_os = "android")]
    pub const RTC_DEVICE_PATH: &str = "/dev/rtc0";

    pub const SYSTEM_UP_TIME_PATH: &str = "/proc/uptime";
}

impl TimeManager {
    #[cfg(target_os = "android")]
    pub fn set_system_clock(msec: i64) -> Result<bool, io::Error> {
        let t = Self::msec_to_timespec(msec);
        debug!("get timespec: {:?}", t);

        // set rtc time
        let mut timerfd;
        let rtc_time = RtcTime::from(time::at_utc(t));
        timerfd = TimerFd::open(ffi::RTC_DEVICE_PATH);
        if let Err(err) = timerfd.set_time(&rtc_time) {
            error!("Failed to set_time(): {}", err);
        }

        // set system time
        unsafe {
            let system_time = timeval {
                tv_sec: t.sec as time_t,
                tv_usec: t.nsec as suseconds_t / 1000,
            };
            settimeofday(&system_time, ptr::null_mut());
        }

        Ok(true)
    }

    // We only change the system time on android
    #[cfg(not(target_os = "android"))]
    pub fn set_system_clock(_msec: i64) -> Result<bool, io::Error> {
        debug!("Running on {} do not change system time", env::consts::OS);
        Ok(true)
    }

    pub fn get_system_clock() -> Result<i64, io::Error> {
        unsafe {
            let mut system_time = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            gettimeofday(&mut system_time, ptr::null_mut());
            let ret: i64 = (system_time.tv_sec as i64 * 1000) + (system_time.tv_usec as i64) / 1000;
            Ok(ret)
        }
    }

    pub fn get_elapsed_real_time() -> Result<i64, io::Error> {
        let ret: Result<f64, io::Error> = TimeInfo::get_elapsed_real_time();
        match ret {
            Ok(t) => {
                debug!("get_elapsed_real_time {:?}", t);

                Ok((t * 1000.0) as i64)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(target_os = "android")]
    pub fn set_timezone(timezone: String) -> Result<bool, io::Error> {
        error!("set timezone {} ", timezone);
        if let Err(err) = AndroidProperties::set("persist.sys.timezone", &timezone) {
            error!("AndroidProperties::set failed {:?}", err);
        }

        Ok(true)
    }

    // We only change the system time on android
    #[cfg(not(target_os = "android"))]
    pub fn set_timezone(_timezone: String) -> Result<bool, io::Error> {
        debug!(
            "Running on {} do not change system timezone",
            env::consts::OS
        );
        Ok(true)
    }

    #[cfg(target_os = "android")]
    fn msec_to_timespec(msec: i64) -> time::Timespec {
        let time_interval: i64 = 1000;
        Timespec::new(
            (msec / time_interval) as i64,
            ((msec % time_interval) * time_interval) as i32,
        )
    }
}

// TimeInfo struct implementation
pub struct TimeInfo {
    elapsed_time: f64,
    #[allow(dead_code)]
    idle_time: f64,
}

impl TimeInfo {
    /// Read a typed value from a sys file.
    pub fn get_elapsed_real_time() -> Result<f64, io::Error> {
        let mut file = fs::File::open(ffi::SYSTEM_UP_TIME_PATH).expect("file not found");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        match Self::from_str(&buffer) {
            Ok(ti) => Ok(ti.elapsed_time),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Bad type")),
        }
    }
}

impl FromStr for TimeInfo {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sarray: Vec<&str> = s
            .trim_matches(|p| p == ' ' || p == '\n')
            .split(' ')
            .collect();

        debug!("from string: [{:?}] [{:?}]", sarray[0], sarray[1]);

        let element1: f64 = sarray[0].parse()?;
        let element2: f64 = sarray[1].parse()?;

        Ok(TimeInfo {
            elapsed_time: element1,
            idle_time: element2,
        })
    }
}
