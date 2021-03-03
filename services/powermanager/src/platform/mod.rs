use crate::PowerManagerSupport;

#[cfg(target_os = "android")]
pub mod android;

// A fallback implementation that never fails.
struct FallbackPowerManager;
impl PowerManagerSupport for FallbackPowerManager {}

/// Returns a trait object implementing PowerManagerSupport for
/// the current platform.
pub fn get_platform_support() -> Box<dyn PowerManagerSupport> {
    #[cfg(target_os = "android")]
    return Box::new(android::AndroidPowerManager::default());

    #[cfg(not(target_os = "android"))]
    return Box::new(FallbackPowerManager);
}
