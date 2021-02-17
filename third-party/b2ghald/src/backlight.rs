/// Manages backlight brightness
/// Todo: discovery of multiple backlights.
use log::error;
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Write};

#[derive(Debug)]
pub struct Backlight {
    path: String,
    max_brightness: u32,
}

impl Backlight {
    pub fn new() -> Result<Self, Error> {
        let reader = read_dir("/sys/class/backlight")?;
        for entry in reader.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let mut sysfs = File::open(path.join("max_brightness"))?;
                let mut content = String::new();
                sysfs.read_to_string(&mut content)?;
                let max_brightness = content.trim().parse::<u32>().unwrap_or(1);
                return Ok(Self {
                    path: path.join("brightness").to_string_lossy().into(),
                    max_brightness,
                });
            }
        }
        error!("No backlight device found.");
        Err(Error::new(ErrorKind::Other, "No backlight support"))
    }

    // Turn on backlight.
    pub fn enable_screen(&self, screen_id: u8) {
        // The only supported screen is #0
        if screen_id != 0 {
            return;
        }
        let _ = self.internal_set_screen_state(screen_id, true);
    }

    // Turn off backlight.
    pub fn disable_screen(&self, screen_id: u8) {
        // The only supported screen is #0
        if screen_id != 0 {
            return;
        }
        let _ = self.internal_set_screen_state(screen_id, false);
    }

    pub fn internal_set_screen_state(&self, screen_id: u8, enabled: bool) -> Result<(), Error> {
        let mut sysfs = OpenOptions::new()
            .write(true)
            .open(format!("/sys/class/graphics/fb{}/blank", screen_id))?;
        // DRM_MODE_DPMS_ON = 0
        // DRM_MODE_DPMS_OFF = 3
        sysfs.write_all(format!("{}", if enabled { 0 } else { 3 }).as_bytes())?;
        sysfs.sync_all()?;
        Ok(())
    }

    // Returns the current brightness in %
    pub fn get_brightness(&self, screen_id: u8) -> u8 {
        self.internal_get_brightness(screen_id).unwrap_or(0)
    }

    fn internal_get_brightness(&self, screen_id: u8) -> Result<u8, Error> {
        // The only supported screen is #0
        if screen_id != 0 {
            return Ok(0);
        }

        let mut sysfs = File::open(&self.path)?;
        let mut content = String::new();
        sysfs.read_to_string(&mut content)?;
        let brightness = content.trim().parse::<u32>().unwrap_or(0);
        Ok((brightness * 100 / self.max_brightness).clamp(0, 100) as _)
    }

    // Sets the brightness in %
    pub fn set_brightness(&self, screen_id: u8, value: u8) {
        let _ = self.internal_set_brightness(screen_id, value);
    }

    pub fn internal_set_brightness(&self, screen_id: u8, value: u8) -> Result<(), Error> {
        // The only supported screen is #0
        if screen_id != 0 {
            return Ok(());
        }

        let value = value.clamp(0, 100);
        let real_value = value as u32 * self.max_brightness / 100;

        let mut sysfs = OpenOptions::new().write(true).open(&self.path)?;
        sysfs.write_all(format!("{}", real_value).as_bytes())?;
        sysfs.sync_all()?;
        Ok(())
    }
}
