#[cfg(all(target_os = "android"))]
mod ffi;

#[cfg(not(target_os = "macos"))]
mod system;

#[cfg(not(target_os = "macos"))]
pub use system::{adjust_process_oom_score, kill_process, set_ext_screen_brightness, SystemState};

use thiserror::Error;

// Returns the amount of memory in MB
pub fn total_memory() -> libc::c_long {
    unsafe {
        libc::sysconf(libc::_SC_PHYS_PAGES) * libc::sysconf(libc::_SC_PAGE_SIZE) / (1024 * 1024)
    }
}

// Dummy implementation used when running tests.
#[cfg(any(not(target_os = "android"), ndk_build))]
mod ffi {
    use std::os::raw::{c_char, c_int};

    pub const PROP_NAME_MAX: usize = 32;
    pub const PROP_VALUE_MAX: usize = 92;

    pub unsafe fn property_get(
        _key: *const c_char,
        _value: *mut c_char,
        _default_value: *const c_char,
    ) -> c_int {
        0
    }

    pub unsafe fn property_get_bool(_key: *const c_char, default_value: i8) -> i8 {
        default_value
    }

    pub unsafe fn property_get_int64(_key: *const c_char, default_value: i64) -> i64 {
        default_value
    }

    pub unsafe fn property_get_int32(_key: *const c_char, default_value: i32) -> i32 {
        default_value
    }

    pub unsafe fn property_set(_key: *const c_char, _value: *const c_char) -> c_int {
        0
    }
}

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

/// Error cases for the property getter and setters.
#[derive(Error, Debug)]
pub enum AndroidPropertyError {
    /// Key longer than PROP_NAME_MAX (32). Will hold the bogus value.
    KeyLength(usize),
    /// Key longer than PROP_VALUE_MAX (92). Will hold the bogus value.
    ValueLength(usize),
    /// Unable to convert this string to a C string.
    InvalidString,
    /// Mirrors the error code returned by the C library.
    Other(c_int),
}

impl std::fmt::Display for AndroidPropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PartialEq for AndroidPropertyError {
    fn eq(&self, other: &AndroidPropertyError) -> bool {
        match (self, other) {
            (AndroidPropertyError::KeyLength(_l1), AndroidPropertyError::KeyLength(_l2)) => true,
            (AndroidPropertyError::ValueLength(_l1), AndroidPropertyError::ValueLength(_l2)) => {
                true
            }
            (AndroidPropertyError::InvalidString, AndroidPropertyError::InvalidString) => true,
            (AndroidPropertyError::Other(c1), AndroidPropertyError::Other(c2)) => c1 == c2,
            (..) => false,
        }
    }
}

/// A generic getter over a default and output type.
/// We use 2 types to allow the <&str, String> case.
pub trait PropertyGetter<I, O> {
    /// Gets the property value for this key, with the expected type.
    fn get(key: &str, default: I) -> Result<O, AndroidPropertyError>;
}

// Turns a Rust string slice into a ffi safe CString.
fn property_string(input: &str) -> Result<CString, AndroidPropertyError> {
    let c_value = CString::new(input);
    if c_value.is_err() {
        return Err(AndroidPropertyError::InvalidString);
    }
    Ok(c_value.unwrap())
}

// Checks that a key fits in Android constraints.
fn check_key_len(key: &str) -> Result<(), AndroidPropertyError> {
    if key.len() >= ffi::PROP_NAME_MAX {
        return Err(AndroidPropertyError::KeyLength(key.len()));
    }
    Ok(())
}

/// The anchor struct for the property getters and setter.
pub struct AndroidProperties;

impl AndroidProperties {
    /// Sets the value of a property.
    pub fn set(key: &str, value: &str) -> Result<(), AndroidPropertyError> {
        check_key_len(key)?;

        if value.len() >= ffi::PROP_VALUE_MAX {
            return Err(AndroidPropertyError::ValueLength(value.len()));
        }

        let c_key = property_string(key)?;
        let c_value = property_string(value)?;
        // Bind pointers outside unsafe block to avoid unexpected deallocation.
        let p_key = c_key.as_ptr();
        let p_value = c_value.as_ptr();

        let res = unsafe { ffi::property_set(p_key, p_value) };

        if res < 0 {
            return Err(AndroidPropertyError::Other(res));
        }

        Ok(())
    }
}

/// bool getter
impl PropertyGetter<bool, bool> for AndroidProperties {
    fn get(key: &str, default: bool) -> Result<bool, AndroidPropertyError> {
        check_key_len(key)?;
        let c_key = property_string(key)?;
        // Bind pointers outside unsafe block to avoid unexpected deallocation.
        let p_key = c_key.as_ptr();
        Ok(unsafe { ffi::property_get_bool(p_key, if default { 1 } else { 0 }) == 1 })
    }
}

/// i32 getter
impl PropertyGetter<i32, i32> for AndroidProperties {
    fn get(key: &str, default: i32) -> Result<i32, AndroidPropertyError> {
        check_key_len(key)?;
        let c_key = property_string(key)?;
        // Bind pointers outside unsafe block to avoid unexpected deallocation.
        let p_key = c_key.as_ptr();
        Ok(unsafe { ffi::property_get_int32(p_key, default) })
    }
}

/// i64 getter
impl PropertyGetter<i64, i64> for AndroidProperties {
    fn get(key: &str, default: i64) -> Result<i64, AndroidPropertyError> {
        check_key_len(key)?;
        let c_key = property_string(key)?;
        // Bind pointers outside unsafe block to avoid unexpected deallocation.
        let p_key = c_key.as_ptr();
        Ok(unsafe { ffi::property_get_int64(p_key, default) })
    }
}

/// String getter.
impl<'a> PropertyGetter<&'a str, String> for AndroidProperties {
    fn get(key: &str, default: &'a str) -> Result<String, AndroidPropertyError> {
        check_key_len(key)?;
        let c_key = property_string(key)?;
        let c_default = property_string(default)?;
        // Bind pointers outside unsafe block to avoid unexpected deallocation.
        let p_key = c_key.as_ptr();
        let p_default = c_default.as_ptr();
        unsafe {
            // Initialize the value with an empty char array of size PROP_VALUE_MAX
            let value = [0u8; ffi::PROP_VALUE_MAX];
            let len = ffi::property_get(p_key, value.as_ptr() as *mut c_char, p_default) as usize;
            String::from_utf8(value[0..len].to_vec())
                .map_err(|_| AndroidPropertyError::InvalidString)
        }
    }
}

#[test]
fn sanity_test() {
    match AndroidProperties::get("this_key_is_way_too_loooooonnnnnnnggggg", false) {
        Err(AndroidPropertyError::KeyLength(len)) => assert_eq!(len, 39),
        _ => panic!("should not reach there!"),
    }

    match AndroidProperties::get("this_key_is_ok", 42i32) {
        Ok(val) => assert_eq!(val, 42),
        _ => panic!("should not reach there!"),
    }

    match AndroidProperties::set("this_key_is_ok", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx") {
        Err(AndroidPropertyError::ValueLength(len)) => assert_eq!(len, 97),
        _ => panic!("should not reach there!"),
    }
}
