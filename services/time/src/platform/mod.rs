#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub use android::time_manager::TimeManager;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::TimeManager;

#[cfg(not(any(target_os = "android", target_os= "linux")))]
mod fallback {
    pub struct FallbackTimeManager;

    impl crate::TimeManagerSupport for FallbackTimeManager {}
}
#[cfg(not(any(target_os = "android", target_os= "linux")))]
pub use fallback::FallbackTimeManager as TimeManager;
