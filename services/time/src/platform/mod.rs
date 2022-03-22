#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub use android::time_manager::TimeManager;

#[cfg(not(target_os = "android"))]
mod fallback {
    pub struct FallbackTimeManager;

    impl crate::TimeManagerSupport for FallbackTimeManager {}
}
#[cfg(not(target_os = "android"))]
pub use fallback::FallbackTimeManager as TimeManager;
