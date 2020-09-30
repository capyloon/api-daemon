mod config;
#[macro_use]
extern crate serde_json;

pub mod generated;
#[macro_use]
pub mod service;

#[cfg(target_os = "android")]
extern crate android_utils;
