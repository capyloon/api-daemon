use std::ffi::CString;
use std::os::raw::{c_char, c_int};

pub enum FotaErr {
    InvalidPath,
    InstallError,
}

#[cfg(target_os = "android")]
mod ffi;

// Dummy implementation used when running tests.
#[cfg(not(target_os = "android"))]
#[allow(unused_variables, non_snake_case)]
mod ffi {
    use std::os::raw::{c_char, c_int};
    pub unsafe fn factoryReset() -> c_int {
        0
    }
    pub unsafe fn installFotaUpdate(updatePath: *const c_char, updatePathLength: c_int) -> c_int {
        0
    }
}

/// The anchor struct for the recovery library.
pub struct AndroidRecovery;

impl AndroidRecovery {
    pub fn factory_reset(_reason: i32) -> i32 {
        // TODO - Bug 81814, support "wipe" and "root" reasons.
        unsafe { ffi::factoryReset() }
    }
    pub fn install_fota_update(update_path: &str) -> Result<(), FotaErr> {
        let cstring_update_path = CString::new(update_path).map_err(|_| FotaErr::InvalidPath)?;
        let c_update_path: *const c_char = cstring_update_path.as_ptr() as *const c_char;
        let c_update_path_length: c_int = update_path.len() as c_int;
        unsafe {
            let install_result = ffi::installFotaUpdate(c_update_path, c_update_path_length);
            if -1 == install_result {
                return Err(FotaErr::InstallError);
            }
        }
        Ok(())
    }
}
