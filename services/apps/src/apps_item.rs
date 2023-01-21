// Internal representation of an application.

use crate::apps_registry::AppsError;
use crate::generated::common::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use url::Host::Domain;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppsItem {
    name: String,
    manifest_url: Url,
    #[serde(default = "AppsItem::default_install_state")]
    install_state: AppsInstallState,
    update_manifest_url: Option<Url>,
    update_url: Option<Url>,
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
    #[serde(default = "AppsItem::default_paths")]
    deeplink_paths: Option<Value>,
}

impl AppsItem {
    pub fn new(name: &str, manifest_url: Url) -> AppsItem {
        AppsItem {
            name: name.into(),
            manifest_url,
            install_state: AppsItem::default_install_state(),
            update_manifest_url: None,
            update_url: None,
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
            deeplink_paths: None,
        }
    }

    pub fn default(name: &str, vhost_port: u16) -> AppsItem {
        let mut app = AppsItem::new(name, AppsItem::new_manifest_url(name, vhost_port));
        app.set_update_manifest_url(AppsItem::new_update_manifest_url(name, vhost_port));
        app
    }

    pub fn default_pwa(name: &str, vhost_port: u16) -> AppsItem {
        let mut app = AppsItem::new(name, AppsItem::new_pwa_url(name, vhost_port));
        app.set_update_manifest_url(AppsItem::new_update_manifest_url(name, vhost_port));
        app
    }

    pub fn set_legacy_manifest_url(&mut self) {
        let url = self.get_manifest_url();
        self.set_manifest_url(url.join("manifest.webapp").unwrap_or(url));
    }

    // Check if the app is a PWA app.
    //   Return:
    //     TRUE: If the manifest URL is http://cached.localhost/*
    //     FALSE: Others.
    pub fn is_pwa(&self) -> bool {
        return self.get_manifest_url().host().unwrap_or(Domain("")) == Domain("cached.localhost");
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

    // Return the URL that will be used by app in web runtime.
    //   In: none
    //   Return:
    //     PWA app: update URL
    //     Package app: manifest URL
    pub fn runtime_url(&self) -> Option<Url> {
        if self.is_pwa() {
            self.get_update_url()
        } else {
            Some(self.get_manifest_url())
        }
    }

    // Return the orign that will be used by app in web runtime.
    //   In: none
    //   Return:
    //     PWA app: the origin of update URL
    //     Package app: the origin manifest URL
    pub fn runtime_origin(&self) -> String {
        if let Some(url) = self.runtime_url() {
            url.origin().unicode_serialization()
        } else {
            String::new()
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_manifest_url(&self) -> Url {
        self.manifest_url.clone()
    }

    pub fn set_removable(&mut self, removable: bool) {
        self.removable = removable;
    }

    pub fn get_update_manifest_url(&self) -> Option<Url> {
        self.update_manifest_url.clone()
    }

    pub fn get_update_url(&self) -> Option<Url> {
        self.update_url.clone()
    }

    pub fn set_manifest_url(&mut self, url: Url) {
        self.manifest_url = url;
    }

    pub fn set_update_manifest_url(&mut self, url: Option<Url>) {
        self.update_manifest_url = url;
    }

    pub fn set_update_url(&mut self, url: Option<Url>) {
        self.update_url = url;
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

    pub fn set_deeplink_paths(&mut self, value: Option<Value>) {
        self.deeplink_paths = value;
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

    pub fn set_manifest_etag(&mut self, etag: Option<String>) {
        self.manifest_etag = etag;
    }

    pub fn get_deeplink_paths(&self) -> Option<Value> {
        self.deeplink_paths.clone()
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

    pub fn is_found(
        &self,
        unique_name: &str,
        update_url: &Option<Url>,
        allow_remove_preloaded: bool,
    ) -> bool {
        let found = self.name == unique_name;
        if self.update_url.is_none() && update_url.is_none() {
            // If the update_url is empty and the removable is true or we
            // explicitely allow to remove preloaded apps,
            // allow the sideload one to override the preload one.
            if allow_remove_preloaded && found {
                false
            } else {
                found && !self.removable
            }
        } else {
            found
        }
    }

    fn default_option() -> Option<String> {
        None
    }

    fn default_paths() -> Option<Value> {
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

    pub fn new_manifest_url(app_name: &str, vhost_port: u16) -> Url {
        if vhost_port == 80 {
            Url::parse(&format!(
                "http://{}.localhost/manifest.webmanifest",
                &app_name
            ))
            // It is safe to unwrap here.
            .unwrap()
        } else {
            Url::parse(&format!(
                "http://{}.localhost:{}/manifest.webmanifest",
                &app_name, vhost_port
            ))
            // It is safe to unwrap here.
            .unwrap()
        }
    }

    pub fn new_pwa_url(app_name: &str, vhost_port: u16) -> Url {
        if vhost_port == 80 {
            Url::parse(&format!(
                "http://cached.localhost/{}/manifest.webmanifest",
                &app_name
            ))
            // It is safe to unwrap here.
            .unwrap()
        } else {
            Url::parse(&format!(
                "http://cached.localhost:{}/{}/manifest.webmanifest",
                vhost_port, &app_name
            ))
            // It is safe to unwrap here.
            .unwrap()
        }
    }

    pub fn new_update_manifest_url(app_name: &str, vhost_port: u16) -> Option<Url> {
        if vhost_port == 80 {
            Url::parse(&format!(
                "http://cached.localhost/{}/update.webmanifest",
                &app_name
            ))
            .ok()
        } else {
            Url::parse(&format!(
                "http://cached.localhost:{}/{}/update.webmanifest",
                vhost_port, &app_name
            ))
            .ok()
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
