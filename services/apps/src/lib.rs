#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod service;
pub mod apps_actor;
pub mod apps_item;
pub mod apps_registry;
pub mod apps_storage;
pub mod apps_utils;
pub mod config;
pub mod downloader;
pub mod generated;
pub mod manifest;
pub mod registry_db;
pub mod shared_state;
pub mod tasks;
pub mod update_manifest;
pub mod update_scheduler;

use crate::shared_state::AppsSharedData;
use common::traits::Shared;
use log::info;
use std::thread;

pub fn start_registry(shared_data: Shared<AppsSharedData>, vhost_port: u16) {
    thread::Builder::new()
        .name("apps service".into())
        .spawn(move || {
            info!("Starting apps service");
            apps_registry::start(shared_data, vhost_port);
        })
        .expect("Failed to start vhost server thread");
    info!("After start apps service");
}
