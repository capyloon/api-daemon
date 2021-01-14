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
    update_manifest_url: String,
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
    #[serde(default = "AppsItem::default_false")]
    removable: bool,
}

impl AppsItem {
    pub fn new(name: &str) -> AppsItem {
        AppsItem {
            name: name.into(),
            manifest_url: AppsItem::default_string(),
            install_state: AppsItem::default_install_state(),
            update_manifest_url: AppsItem::default_string(),
            update_url: AppsItem::default_string(),
            status: AppsItem::default_status(),
            update_state: AppsItem::default_update_state(),
            install_time: AppsItem::default_time(),
            update_time: AppsItem::default_time(),
            manifest_hash: AppsItem::default_string(),
            package_hash: AppsItem::default_string(),
            version: AppsItem::default_string(),
            removable: true,
        }
    }

    pub fn default(name: &str, vhost_port: u16) -> AppsItem {
        let mut app = AppsItem::new(name);
        app.set_manifest_url(&AppsItem::new_manifest_url(name, vhost_port));
        app.set_update_manifest_url(&AppsItem::new_update_manifest_url(name, vhost_port));
        app
    }

    pub fn default_pwa(name: &str, vhost_port: u16) -> AppsItem {
        let mut app = AppsItem::new(name);
        app.set_manifest_url(&AppsItem::new_pwa_url(name, vhost_port));
        app.set_update_manifest_url(&AppsItem::new_update_manifest_url(name, vhost_port));
        app
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_manifest_url(&mut self, manifest_url: &str) {
        self.manifest_url = manifest_url.into();
    }

    pub fn get_manifest_url(&self) -> String {
        self.manifest_url.clone()
    }

    pub fn set_removable(&mut self, removable: bool) {
        self.removable = removable;
    }

    pub fn get_update_manifest_url(&self) -> String {
        self.update_manifest_url.clone()
    }

    pub fn get_update_url(&self) -> String {
        self.update_url.clone()
    }

    pub fn set_update_manifest_url(&mut self, url: &str) {
        self.update_manifest_url = url.into();
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

    pub fn get_removable(&self) -> bool {
        self.removable
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

    fn default_false() -> bool {
        false
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

    pub fn new_manifest_url(app_name: &str, vhost_port: u16) -> String {
        if vhost_port == 80 {
            format!("http://{}.localhost/manifest.webmanifest", &app_name)
        } else {
            format!("http://{}.localhost:{}/manifest.webmanifest", &app_name, vhost_port)
        }
    }

    pub fn new_pwa_url(app_name: &str, vhost_port: u16) -> String {
        if vhost_port == 80 {
            format!("http://cached.localhost/{}/manifest.webmanifest", &app_name)
        } else {
            format!(
                "http://cached.localhost:{}/{}/manifest.webmanifest",
                vhost_port, &app_name
            )
        }
    }

    pub fn new_update_manifest_url(app_name: &str, vhost_port: u16) -> String {
        if vhost_port == 80 {
            format!("http://cached.localhost/{}/update.webmanifest", &app_name)
        } else {
            format!(
                "http://cached.localhost:{}/{}/update.webmanifest",
                vhost_port, &app_name
            )
        }
    }
}

impl From<&AppsItem> for AppsObject {
    fn from(app: &AppsItem) -> Self {
        AppsObject {
            name: app.name.clone(),
            install_state: app.install_state,
            manifest_url: app.manifest_url.clone(),
            removable: app.removable,
            status: app.status,
            update_state: app.update_state,
            update_url: app.update_url.clone(),
            update_manifest_url: app.update_manifest_url.clone(),
            allowed_auto_download: false,
        }
    }
}
