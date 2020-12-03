use std::ffi::CString;

#[cfg(target_os = "android")]
mod ffi;

// Dummy implementation used when running tests.
#[cfg(not(target_os = "android"))]
#[allow(unused_variables)]
mod ffi {
    use std::os::raw::{c_char, c_int};
    pub unsafe fn acquire_wake_lock(lock: c_int, id: *const c_char) -> c_int {
        0
    }
    pub unsafe fn release_wake_lock(id: *const c_char) -> c_int {
        0
    }
}

/// The anchor struct for the power library.
pub struct AndroidPower;

impl AndroidPower {
    pub fn acquire_wake_lock(lock: i32, id: &str) -> Result<(), String> {
        let cstring_id = CString::new(id).expect("Failed to create CString.");
        match unsafe { ffi::acquire_wake_lock(lock, cstring_id.as_ptr() as _) } {
            0 => Ok(()),
            _ => Err("Failed to acquire wake lock from android suspend service".into())
        }
    }

    pub fn release_wake_lock(id: &str) -> Result<(), String> {
        let cstring_id = CString::new(id).expect("Failed to create CString.");
        match unsafe { ffi::release_wake_lock(cstring_id.as_ptr() as _) } {
            0 => Ok(()),
            _ => Err("Failed to release wake lock from android suspend service".into())
        }
    }
}
