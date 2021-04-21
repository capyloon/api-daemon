mod cgroups;
pub mod generated;
pub mod service;
#[cfg(test)]
mod tests;

#[cfg(target_os = "android")]
mod android_worker;
#[cfg(not(target_os = "android"))]
mod fallback_worker;

#[cfg(target_os = "android")]
pub(crate) type WorkerType = crate::android_worker::CGroupsWorker;
#[cfg(not(target_os = "android"))]
pub(crate) type WorkerType = crate::fallback_worker::CGroupsWorker;
