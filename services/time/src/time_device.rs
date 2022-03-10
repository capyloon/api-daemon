use libc::c_int;
use log::{error, info};
use std::os::unix::io::AsRawFd;
use std::{fs, io, path};
use time::*;

mod ffi {
    use super::RtcTime;
    use super::Timespec;
    use nix::{ioctl_read_buf, ioctl_write_buf};

    const ANDROID_ALARM_MAGIC: u8 = b'a';
    const ANDROID_ALARM_GET_TIME: u8 = 4;
    const ANDROID_ALARM_SET_RTC: u8 = 5;

    const RTC_MAGIC: u8 = b'p';
    const RTC_RD_TIME: u8 = 0x09;
    const RTC_SET_TIME: u8 = 0x0a;

    ioctl_read_buf!(
        alarm_get_clock,
        ANDROID_ALARM_MAGIC,
        ANDROID_ALARM_GET_TIME,
        Timespec
    );
    ioctl_write_buf!(
        alarm_set_clock,
        ANDROID_ALARM_MAGIC,
        ANDROID_ALARM_SET_RTC,
        Timespec
    );

    ioctl_read_buf!(rtc_rd_time, RTC_MAGIC, RTC_RD_TIME, RtcTime);
    ioctl_write_buf!(rtc_set_time, RTC_MAGIC, RTC_SET_TIME, RtcTime);
}

pub const YEAR_EPOCH: i32 = 1900;

/**
 * RtcTime implementation
 */
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct RtcTime {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
}

impl RtcTime {
    pub fn new() -> Self {
        RtcTime {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
        }
    }
}

impl From<time::Tm> for RtcTime {
    fn from(tm: time::Tm) -> RtcTime {
        RtcTime {
            tm_sec: tm.tm_sec as i32,
            tm_min: tm.tm_min as i32,
            tm_hour: tm.tm_hour as i32,
            tm_mday: tm.tm_mday as i32,
            tm_mon: tm.tm_mon as i32,
            tm_year: tm.tm_year as i32,
            tm_wday: tm.tm_wday as i32,
            tm_yday: tm.tm_yday as i32,
            tm_isdst: tm.tm_isdst as i32,
        }
    }
}

/**
 * TimeDevice trait
 */
pub trait TimeDevice {
    type TimeFormat;

    fn open<P: AsRef<path::Path>>(path: P) -> Result<Self, io::Error>
    where
        Self: Sized;

    fn get_time(&mut self) -> Result<Self::TimeFormat, nix::Error>;

    fn set_time(&mut self, time: &Self::TimeFormat) -> Result<i32, nix::Error>;
}

/**
 * TimerFd struct implementation
 */
#[derive(Debug)]
pub struct TimerFd {
    dev: fs::File,
}

impl TimeDevice for TimerFd {
    type TimeFormat = RtcTime;

    fn open<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<path::Path>,
    {
        match fs::File::open(path) {
            Ok(dev) => Ok(TimerFd { dev }),
            Err(err) => {
                error!("Failed to open error: {:?}", err);
                Err(err)
            }
        }
    }

    fn get_time(&mut self) -> Result<RtcTime, nix::Error> {
        let mut rt: [RtcTime; 1] = [RtcTime::new(); 1];
        unsafe {
            let _ = ffi::rtc_rd_time(self.dev.as_raw_fd(), &mut rt);
        }

        Ok(rt[0])
    }

    fn set_time(&mut self, rt: &RtcTime) -> Result<i32, nix::Error> {
        info!("rtc device set time {:?}", rt);
        unsafe { ffi::rtc_set_time(self.dev.as_raw_fd(), &[*rt]) }
    }
}

/**
 * AlarmDriver struct implementation
 */
#[derive(Debug)]
pub struct AlarmDriver {
    dev: fs::File,
}

impl TimeDevice for AlarmDriver {
    type TimeFormat = Timespec;

    fn open<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<path::Path>,
    {
        match fs::File::open(path) {
            Ok(dev) => Ok(AlarmDriver { dev }),
            Err(err) => {
                error!("Failed to open error: {:?}", err);
                Err(err)
            }
        }
    }

    fn get_time(&mut self) -> Result<Timespec, nix::Error> {
        // create timespec object
        let mut ts: [Timespec; 1] = [Timespec::new(0, 0); 1];
        info!("alarm driver get time");
        assert_eq!(0, unsafe {
            ffi::alarm_get_clock(self.dev.as_raw_fd(), &mut ts)
        }?);

        Ok(ts[0])
    }

    fn set_time(&mut self, ts: &Timespec) -> Result<i32, nix::Error> {
        info!("alarm driver set time {:?}", ts);
        unsafe { ffi::alarm_set_clock(self.dev.as_raw_fd(), &[*ts]) }
    }
}
