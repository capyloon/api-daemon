// Internal representation of an application.

use crate::apps_registry::AppsError;
use crate::generated::common::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use url::Host::Domain;
use url::Url;

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
    #[serde(default = "AppsItem::default_false")]
    preloaded: bool,
    #[serde(default = "AppsItem::default_option")]
    manifest_etag: Option<String>,
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
            preloaded: false,
            manifest_etag: None,
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

    // Check if the app is a PWA app.
    //   Return:
    //     TRUE: If the manifest URL is http://cached.localhost/*
    //     FALSE: Others.
    pub fn is_pwa(&self) -> bool {
        if let Ok(manifest_url) = Url::parse(&self.get_manifest_url()) {
            return manifest_url.host().unwrap_or(Domain("")) == Domain("cached.localhost");
        }
        false
    }

    // Return the storage path of the app to load the manifest file.
    //   In:
    //     config_path: The data path in the config file.
    //   Return:
    //     PWA app: {config_path}/cached/{app-name}
    //     Package app: {config_path}/vroot/{app-name}
    pub fn get_appdir(&self, config_path: &Path) -> Result<PathBuf, AppsError> {
        if self.is_pwa() {
            Ok(config_path.join("cached").join(&self.name))
        } else {
            Ok(config_path.join("vroot").join(&self.name))
        }
    }

    // Return the orign that will be used by app in web runtime.
    //   In: none
    //   Return:
    //     PWA app: update URL
    //     Package app: manifest URL
    pub fn runtime_origin(&self) -> String {
        let url = if self.is_pwa() {
            self.get_update_url()
        } else {
            self.get_manifest_url()
        };
        if let Ok(url) = Url::parse(&url) {
            url.origin().unicode_serialization()
        } else {
            String::new()
        }
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

    pub fn set_manifest_etag_str(&mut self, etag: &str) {
        self.manifest_etag = if etag.is_empty() {
            None
        } else {
            Some(etag.into())
        };
    }

    pub fn set_manifest_etag(&mut self, etag: Option<String>) {
        self.manifest_etag = etag;
    }

    pub fn get_manifest_etag(&self) -> Option<String> {
        self.manifest_etag.clone()
    }

    pub fn set_package_hash(&mut self, hash: &str) {
        self.package_hash = hash.to_owned();
    }

    pub fn set_preloaded(&mut self, preloaded: bool) {
        self.preloaded = preloaded;
    }

    pub fn get_preloaded(&self) -> bool {
        self.preloaded
    }

    pub fn is_found(&self, unique_name: &str, update_url: Option<&str>) -> bool {
        let found = self.name == unique_name;
        if self.update_url.is_empty() && update_url.is_none() {
            // If the update_url is empty and the removable is true,
            // allow the sideload one to override the preload one.
            found && !self.removable
        } else {
            found
        }
    }

    fn default_option() -> Option<String> {
        None
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
            format!(
                "http://{}.localhost:{}/manifest.webmanifest",
                &app_name, vhost_port
            )
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
            preloaded: app.preloaded,
            progress: 0,
            origin: app.runtime_origin(),
        }
    }
}
