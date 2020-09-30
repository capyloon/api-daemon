/// Helper functions to deal with kernel versions.
use crate::selinux::{SeLinux, SeLinuxEnforceState};
#[cfg(not(test))]
use android_utils::{AndroidProperties, PropertyGetter};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};

#[cfg(test)]
mod mock_android_prop {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    pub struct AndroidProperties;

    impl AndroidProperties {
        pub fn get(_key: &str, _default: &str) -> Result<String, ()> {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);

            match CALL_COUNT.load(Ordering::SeqCst) {
                1 => Ok("invalid build".into()),
                2 => Ok("QC_8905_3_10_49_SEC_1".into()),
                _ => Err(())
            }
        }
    }
}
#[cfg(test)]
use mock_android_prop::AndroidProperties;

// Return true if the device runs a known-good build and has expected SELinux enforcement status.
pub fn check_system_state(needs_selinux: bool,
                          white_list: Option<&str>) -> Result<bool, Error> {
    // First check if the ro.build.cver is part of the whitelist.
    if let Ok(build_prop) = AndroidProperties::get("ro.build.cver", "") {
        if build_prop.is_empty() {
            return Ok(false);
        }

        if let Some(lists) = white_list {
            if lists.split('\n')
                .map(|item| item.trim())
                .find(|item| build_prop == *item)
                .is_none()
            {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    } else {
        return Ok(false);
    }

    // And now also check that the SELinux state matches expectations.
    match SeLinux::getenforce() {
        Ok(level) => {
            let res = {
                if needs_selinux {
                    SeLinuxEnforceState::Enforcing == level
                } else {
                    true
                }
            };
            Ok(res)
        }
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("SELinux error: {}", err),
        )),
    }
}

// Returns the SELinux enforcing status and the Kernel version as read from /proc/version
pub fn system_info() -> Result<(SeLinuxEnforceState, String), Error> {
    let mut file = File::open("/proc/version")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    // We expect a string like:
    // Linux version 4.9.186 (fabrice@builder) (gcc version 4.9.x 20150123 (prerelease) (GCC) ) #1 SMP PREEMPT Wed Jan 22 11:56:13 PST 2020
    // If the string doesn't start with "Linux version" something is fishy.
    if !buffer.starts_with("Linux version") {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Invalid /proc/version content: {}", buffer),
        ));
    }

    let parts: Vec<&str> = buffer.split(' ').collect();
    if parts.len() < 3 {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "Invalid /proc/version content: no version found ({})",
                buffer
            ),
        ));
    }

    match SeLinux::getenforce() {
        Ok(level) => Ok((level, parts[2].into())),
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("SELinux error: {}", err),
        )),
    }
}

pub struct KernelVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl KernelVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    // Return true if this version is compatible with another one.
    // This is the case if and only if major and minor version are
    // equal, and if the patch version of the current object is greater
    // or equal to the patch version of the other kernel:
    // 3.10.4 compatible_with(3.10.2) -> false
    // 3.10.4 compatible_with(3.10.5) -> true
    // 3.10.4 compatible_with(3.9.3) -> false
    // 3.10.4 compatible_with(4.4.2) -> false
    pub fn compatible_with(&self, other: &KernelVersion) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch >= other.patch
    }
}

use std::convert::TryFrom;

impl TryFrom<&String> for KernelVersion {
    type Error = &'static str;

    fn try_from(source: &String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = source.split('.').collect();

        if parts.len() != 3 {
            return Err("Invalid input");
        }

        let major = parts[0].parse::<u16>().map_err(|_| "Invalid input")?;
        let minor = parts[1].parse::<u16>().map_err(|_| "Invalid input")?;
        let patch = parts[2].parse::<u16>().map_err(|_| "Invalid input")?;

        Ok(Self::new(major, minor, patch))
    }
}

pub fn system_matches(
    expected_selinux: SeLinuxEnforceState,
    valid_kernels: Vec<KernelVersion>,
) -> bool {
    // Test configuration: Enforcing SELinux, kernel 3.10.49
    #[cfg(test)]
    let system_info: Result<(SeLinuxEnforceState, String), Error> =
        Ok((SeLinuxEnforceState::Enforcing, "3.10.49".into()));

    #[cfg(not(test))]
    let system_info = system_info();

    if let Ok(info) = system_info {
        if info.0 != expected_selinux {
            return false;
        }

        if let Ok(kernel_version) = KernelVersion::try_from(&info.1) {
            // Check if we match one the provided kernels.
            for kernel in valid_kernels {
                if kernel_version.compatible_with(&kernel) {
                    return true;
                }
            }
        }
    }

    false
}

#[test]
fn check_system_info() {
    // Note: This is testing the CI runners configuration.
    // Update as needed if it changes.
    // Not all the CI runners run the same kernel, so we can't check the version,
    // but not failing to unwrap() system_info() is good enough.
    let info = system_info().unwrap();
    assert_eq!(info.0, SeLinuxEnforceState::Disabled);
}

#[test]
fn check_kernel_version() {
    // The test kernel version is set to 3.10.49 in system_matches()

    // Succeeds because 3.10.49 is compatible with the expectation of 3.10.48
    assert!(system_matches(
        SeLinuxEnforceState::Enforcing,
        vec![KernelVersion::new(3, 10, 48), KernelVersion::new(4, 10, 8)]
    ));

    // Fails because of the mismatch in SELinux status.
    assert!(!system_matches(
        SeLinuxEnforceState::Disabled,
        vec![KernelVersion::new(3, 10, 48), KernelVersion::new(4, 10, 8)]
    ));

    // Fails to match if the expected patch for this X.Y is too high, or
    // if we expect a different M.N
    assert!(!system_matches(
        SeLinuxEnforceState::Enforcing,
        vec![KernelVersion::new(3, 10, 50), KernelVersion::new(2, 10, 8)]
    ));
}

#[test]
fn system_state() {
    let mut file = File::open("test-fixtures/valid_build_props.txt").unwrap();
    let mut lists = String::new();
    file.read_to_string(&mut lists);

    // First call will fail because the ro.build property is set to "invalid build".
    let res = check_system_state(false, Some(&lists));
    assert_eq!(res.unwrap(), false);

    // Second call will success because the ro.build property is in the list.
    let res = check_system_state(false, Some(&lists));
    assert_eq!(res.unwrap(), true);
}
