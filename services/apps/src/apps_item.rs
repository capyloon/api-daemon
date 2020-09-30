// Internal representation of an application.

use crate::generated::common::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppsItem {
    name: String,
    manifest_url: String,
    #[serde(default = "AppsItem::default_install_state")]
    install_state: AppsInstallState,
    #[serde(default = "AppsItem::default_string")]
    update_url: String,
    #[serde(default = "AppsItem::default_status")]
    status: AppsStatus,
    #[serde(default = "AppsItem::default_update_state")]
    update_state: AppsUpdateState,
    #[serde(default = "AppsItem::default_time")]
    install_time: u64,
    #[serde(default = "AppsItem::default_time")]
    update_time: u64,
    #[serde(default = "AppsItem::default_string")]
    manifest_hash: String,
    #[serde(default = "AppsItem::default_string")]
    package_hash: String,
    #[serde(default = "AppsItem::default_string")]
    version: String,
}

impl AppsItem {
    pub fn default(name: &str, manifest_url: &str) -> AppsItem {
        AppsItem {
            name: name.into(),
            manifest_url: manifest_url.into(),
            install_state: AppsItem::default_install_state(),
            update_url: AppsItem::default_string(),
            status: AppsItem::default_status(),
            update_state: AppsItem::default_update_state(),
            install_time: AppsItem::default_time(),
            update_time: AppsItem::default_time(),
            manifest_hash: AppsItem::default_string(),
            package_hash: AppsItem::default_string(),
            version: AppsItem::default_string(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_manifest_url(&self) -> String {
        self.manifest_url.clone()
    }

    pub fn get_update_url(&self) -> String {
        self.update_url.clone()
    }

    pub fn set_update_url(&mut self, url: &str) {
        self.update_url = url.into();
    }

    pub fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }

    pub fn get_install_time(&self) -> u64 {
        self.install_time
    }

    pub fn get_update_time(&self) -> u64 {
        self.update_time
    }

    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    pub fn get_status(&self) -> AppsStatus {
        self.status
    }

    pub fn get_install_state(&self) -> AppsInstallState {
        self.install_state
    }

    pub fn get_update_state(&self) -> AppsUpdateState {
        self.update_state
    }

    pub fn get_manifest_hash(&self) -> String {
        self.manifest_hash.clone()
    }

    pub fn get_package_hash(&self) -> String {
        self.package_hash.clone()
    }

    pub fn set_status(&mut self, status: AppsStatus) {
        self.status = status;
    }

    pub fn set_install_state(&mut self, state: AppsInstallState) {
        self.install_state = state;
    }

    pub fn set_update_state(&mut self, state: AppsUpdateState) {
        self.update_state = state;
    }

    pub fn set_install_time(&mut self, time: u64) {
        self.install_time = time;
    }

    pub fn set_update_time(&mut self, time: u64) {
        self.update_time = time;
    }

    pub fn set_manifest_hash(&mut self, hash: &str) {
        self.manifest_hash = hash.to_owned();
    }

    pub fn set_package_hash(&mut self, hash: &str) {
        self.package_hash = hash.to_owned();
    }

    fn default_string() -> String {
        String::new()
    }

    fn default_update_state() -> AppsUpdateState {
        AppsUpdateState::Idle
    }

    fn default_install_state() -> AppsInstallState {
        AppsInstallState::Installed
    }

    fn default_status() -> AppsStatus {
        AppsStatus::Enabled
    }

    fn default_time() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis() as u64,
            Err(_) => 0,
        }
    }
}

impl From<&AppsItem> for AppsObject {
    fn from(app: &AppsItem) -> Self {
        AppsObject {
            name: app.name.clone(),
            install_state: app.install_state,
            manifest_url: app.manifest_url.clone(),
            status: app.status,
            update_state: app.update_state,
            update_url: app.update_url.clone(),
            allowed_auto_download: false,
        }
    }
}
