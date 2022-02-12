/// A simple selinux wrapper.

#[cfg(not(target_os = "macos"))]
mod ffi {
    use std::os::raw::c_int;

    #[link(name = "selinux")]
    extern "C" {
        #[cfg(target_os = "android")]
        pub fn setcon(context: *const libc::c_char) -> c_int;
        pub fn security_getenforce() -> c_int;
    }
}

#[derive(Debug, PartialEq)]
pub enum SeLinuxEnforceState {
    Permissive,
    Enforcing,
    Disabled,
}

pub struct SeLinux {}

impl SeLinux {
    #[cfg(target_os = "android")]
    pub fn setcon(context: &str) -> bool {
        use std::ffi::CString;

        let context = CString::new(context).expect("CString::new failed");
        let res = unsafe { ffi::setcon(context.as_ptr()) };
        res == 0
    }

    #[cfg(not(target_os = "macos"))]
    pub fn getenforce() -> Result<SeLinuxEnforceState, String> {
        let res = unsafe { ffi::security_getenforce() };

        match res {
            0 => Ok(SeLinuxEnforceState::Permissive),
            1 => Ok(SeLinuxEnforceState::Enforcing),
            -1 => Ok(SeLinuxEnforceState::Disabled),
            _ => Err(format!(
                "Unexpected value returned by security_getenforce() : {}",
                res
            )),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn getenforce() -> Result<SeLinuxEnforceState, String> {
        Ok(SeLinuxEnforceState::Enforcing)
    }
}
