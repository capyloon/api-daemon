use crate::PowerManagerSupport;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "linux")]
pub mod linux;

// A fallback implementation that never fails.
struct FallbackPowerManager;
impl PowerManagerSupport for FallbackPowerManager {}

/// Returns a trait object implementing PowerManagerSupport for
/// the current platform.
pub fn get_platform_support() -> Box<dyn PowerManagerSupport> {
    #[cfg(target_os = "android")]
    return Box::new(android::AndroidPowerManager::default());

    #[cfg(target_os = "linux")]
    {
        if let Some(mut linux_pm) = linux::LinuxPowerManager::new() {
            linux_pm.set_screen_state(true, 0);
            return Box::new(linux_pm);
        }
    }

    // Fallback for error cases and unsupported platforms.
    return Box::new(FallbackPowerManager);
}
