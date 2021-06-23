/// Registry for apps services:
/// - Init app from system to data partition.
/// - Maintain the runtime set of apps.
/// - Exposes high level apis to manipulate the registry.
use crate::apps_actor;
use crate::apps_item::AppsItem;
use crate::apps_request::is_new_version;
use crate::apps_storage::AppsStorage;
use crate::config::Config;
use crate::downloader::DownloadError;
use crate::generated::common::*;
use crate::manifest::{Manifest, ManifestError};
use crate::registry_db::RegistryDb;
use crate::service::AppsService;
use crate::shared_state::AppsSharedData;
use crate::update_scheduler;
use android_utils::{AndroidProperties, PropertyGetter};
use common::traits::{DispatcherId, Shared};
use common::JsonValue;
use log::{debug, error, info};
use serde_json::{json, Value};
use settings_service::db::{DbObserver, ObserverType};
use settings_service::generated::common::SettingInfo;
use settings_service::service::SettingsService;
use std::collections::hash_map::DefaultHasher;
use std::convert::From;
use std::env;
use std::fs;
use std::fs::{remove_dir_all, remove_file, File};
use std::hash::{Hash, Hasher};
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use threadpool::ThreadPool;
use url::ParseError;

#[cfg(test)]
use crate::apps_storage::validate_package;

// Relay the request to Gecko using the bridge.
use common::traits::Service;
use geckobridge::service::GeckoBridgeService;

#[derive(Error, Debug)]
pub enum AppsError {
    #[error("Custom Error")]
    AppsConfigError,
    #[error("AppsError")]
    WrongManifest,
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Zip Error")]
    Url(#[from] ParseError),
    #[error("Url Error")]
    Zip(#[from] zip::result::ZipError),
    #[error("RegistryDb error")]
    RegistryDb(#[from] crate::registry_db::Error),
    #[error("No db available")]
    NoDb,
}

#[derive(Error, Debug)]
pub enum AppsMgmtError {
    #[error("AppsMgmtError")]
    DiskSpaceNotEnough,
    #[error("ManifestDownloadFailed {:?}", 0)]
    ManifestDownloadFailed(DownloadError),
    #[error("AppsMgmtError")]
    ManifestReadFailed,
    #[error("AppsMgmtError")]
    ManifestInvalid,
    #[error("AppsMgmtError")]
    DownloaderError,
    #[error("AppsMgmtError")]
    DownloadFailed,
    #[error("AppsMgmtError")]
    DownloadNotModified,
    #[error("AppsMgmtError")]
    PackageCorrupt,
    #[error("PackageDownloadFailed {:?}", 0)]
    PackageDownloadFailed(DownloadError),
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
}

impl PartialEq for AppsMgmtError {
    fn eq(&self, right: &AppsMgmtError) -> bool {
        format!("{:?}", self) == format!("{:?}", right)
    }
}

#[derive(Error, Debug)]
pub enum RegistrationError {
    #[error("RegistrationError Manifest Error, `{0}`")]
    WrongManifest(#[from] ManifestError),
    #[error("RegistrationError Manifest URL Missing")]
    ManifestUrlMissing,
    #[error("RegistrationError Update URL Missing")]
    UpdateUrlMissing,
    #[error("RegistrationError Manifest URL Not Found")]
    ManifestURLNotFound,
    #[error("RegistrationError AppsError, `{0}`")]
    WrongApp(#[from] AppsError),
}

pub fn hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
struct SettingObserver {}

impl DbObserver for SettingObserver {
    fn callback(&self, name: &str, value: &JsonValue) {
        if name != "language.current" {
            error!(
                "unexpected key {} / value {}",
                name,
                value.as_str().unwrap_or("unknown")
            );
            return;
        }

        let lang = value.as_str().unwrap_or("en-US");
        AppsService::shared_state().lock().registry.set_lang(lang);
    }
}

#[derive(Default)]
pub struct AppsRegistry {
    pool: ThreadPool,       // The thread pool used to run tasks.
    db: Option<RegistryDb>, // The sqlite DB wrapper.
    vhost_port: u16,        // Keeping vhost vhost_port number in registry
    lang: String,
    apps_list: Vec<AppsObject>,
    pub event_broadcaster: AppsEngineEventBroadcaster,
}

impl AppsRegistry {
    pub fn add_dispatcher(&mut self, dispatcher: &AppsEngineEventDispatcher) -> DispatcherId {
        self.event_broadcaster.add(dispatcher)
    }

    pub fn remove_dispatcher(&mut self, id: DispatcherId) {
        self.event_broadcaster.remove(id)
    }

    pub fn broadcast_installing(&self, is_update: bool, apps_object: AppsObject) {
        if is_update {
            self.event_broadcaster.broadcast_app_updating(apps_object);
        } else {
            self.event_broadcaster.broadcast_app_installing(apps_object);
        }
    }

    pub fn restore_apps_status(&mut self, is_update: bool, apps_item: &AppsItem) {
        if is_update {
            let _ = self.save_app(is_update, &apps_item);
        } else {
            let _ = self.unregister(&apps_item.get_manifest_url());
        }
    }

    pub fn initialize(config: &Config, vhost_port: u16) -> Result<Self, AppsError> {
        let current = env::current_dir().unwrap();
        let root_dir = current.join(config.root_path.clone());
        let data_dir = current.join(config.data_path.clone());

        // Make sure the directory structure is set properly.
        let installed_dir = data_dir.join("installed");
        if !AppsStorage::ensure_dir(&installed_dir) {
            error!(
                "Failed to ensure that the {} directory exists.",
                installed_dir.display()
            );
            return Err(AppsError::AppsConfigError);
        }

        let cached_dir = data_dir.join("cached");
        if !AppsStorage::ensure_dir(&cached_dir) {
            error!(
                "Failed to ensure that the {} directory exists.",
                cached_dir.display()
            );
            return Err(AppsError::AppsConfigError);
        }

        let db_dir = data_dir.join("db");
        if !AppsStorage::ensure_dir(db_dir.as_path()) {
            error!(
                "Failed to ensure that the {} directory exists.",
                db_dir.display()
            );
            return Err(AppsError::AppsConfigError);
        }

        let vroot_dir = data_dir.join("vroot");
        if !AppsStorage::ensure_dir(vroot_dir.as_path()) {
            error!(
                "Failed to ensure that the {} directory exists.",
                vroot_dir.display()
            );
            return Err(AppsError::AppsConfigError);
        }

        // Open the registry database.
        let mut db = RegistryDb::new(db_dir.join(&"apps.sqlite"))?;
        let count = db.count()?;

        // No apps yet, load the default ones from the json file.
        if count == 0 {
            info!("Initializing apps registry from {}", config.root_path);

            let webapps_json = root_dir.join("webapps.json");
            match AppsStorage::read_webapps(webapps_json) {
                Ok(mut apps) => {
                    for app in &mut apps {
                        if let Ok(app) = AppsStorage::add_app(app, config, vhost_port) {
                            let _ = db.add(&app)?;
                        }
                    }
                }
                Err(err) => {
                    error!("No apps were found: {:?}", err);
                }
            }

            // Create a symbolic link for cached in vroot.
            let dest = vroot_dir.join("cached");
            let _ = symlink(&cached_dir, &dest);
        }

        let apps_list = match db.get_all() {
            Ok(apps) => apps.iter().map(AppsObject::from).collect(),
            Err(_) => vec![],
        };

        let setting_service = SettingsService::shared_state();
        let lang = match setting_service.lock().db.get("language.current") {
            Ok(value) => value.as_str().unwrap_or("en-US").to_string(),
            Err(_) => "en-US".to_string(),
        };
        // the life time of AppsRegistry is the same as the process. We don't need to remove_observer
        let id = setting_service.lock().db.add_observer(
            "language.current",
            ObserverType::FuncPtr(Box::new(SettingObserver {})),
        );
        info!("add_observer to SettingsService with id {}", id);

        Ok(Self {
            pool: ThreadPool::new(5),
            db: Some(db),
            vhost_port,
            lang,
            apps_list,
            event_broadcaster: AppsEngineEventBroadcaster::default(),
        })
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.lang = lang.to_string();
    }

    pub fn get_lang(&self) -> String {
        self.lang.clone()
    }
    // Returns a sanitized version of the name, usable as:
    // - a filename
    // - a subdomain
    // The result is ASCII only with no whitespace.
    pub fn sanitize_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }

    // Get a unique name for the app of a given update_url.
    // In:
    //    name - the app name in the app manifest.
    //    update_url - the update_url of an app.
    // return:
    //    app_name - name or name + [1-999] for different update_url.
    pub fn get_unique_name(
        &self,
        name: &str,
        update_url: Option<&str>,
    ) -> Result<String, AppsServiceError> {
        if let Some(db) = &self.db {
            if let Some(url) = update_url {
                // The update_url is unique return name from the db.
                if let Ok(app) = db.get_by_update_url(url) {
                    return Ok(app.get_name());
                }
            }

            let sanitized_name: String = Self::sanitize_name(name);
            let mut unique_name = sanitized_name.clone();
            if unique_name.is_empty() {
                return Err(AppsServiceError::InvalidAppName);
            }

            // "cached" is reserved for pwa apps.
            if unique_name == "cached" {
                return Err(AppsServiceError::InvalidAppName);
            }
            // For new update_url, return name or name + [1-999]
            if let Ok(apps) = db.get_all() {
                let mut count = 1;
                loop {
                    if apps
                        .iter()
                        .find(|app| app.is_found(&unique_name, update_url))
                        .is_none()
                    {
                        break;
                    }
                    if count > 999 {
                        return Err(AppsServiceError::InvalidAppName);
                    }
                    unique_name = format!("{}{}", sanitized_name, count);
                    count += 1;
                }
            }
            Ok(unique_name)
        } else {
            Err(AppsServiceError::InvalidState)
        }
    }

    pub fn save_app(
        &mut self,
        is_update: bool,
        apps_item: &AppsItem,
    ) -> Result<(), AppsServiceError> {
        if is_update {
            self.update_app(apps_item)
                .map_err(|_| AppsServiceError::RegistrationError)?
        } else {
            self.register_app(apps_item)
                .map_err(|_| AppsServiceError::RegistrationError)?
        }

        Ok(())
    }

    pub fn clear(
        &mut self,
        manifest_url: &str,
        data_type: ClearType,
        data_path: &str,
    ) -> Result<(), AppsServiceError> {
        let type_str = match data_type {
            ClearType::Browser => "Browser",
            ClearType::Storage => "Storage",
        };

        let features = self.get_b2g_features(manifest_url, data_path);

        Ok(GeckoBridgeService::shared_state()
            .lock()
            .apps_service_on_clear(manifest_url.to_string(), type_str.to_string(), features)
            .map_err(|_| AppsServiceError::ClearDataError)?)
    }

    pub fn get_pacakge_path(
        &self,
        webapp_dir: &str,
        manifest_url: &str,
    ) -> Result<PathBuf, AppsServiceError> {
        if let Some(app) = self.get_by_manifest_url(manifest_url) {
            let webapp_path = Path::new(&webapp_dir);
            let app_path = webapp_path.join("vroot").join(&app.get_name());
            let package_path = app_path.join("application.zip");

            if File::open(&package_path).is_ok() {
                Ok(package_path)
            } else {
                Err(AppsServiceError::AppNotFound)
            }
        } else {
            Err(AppsServiceError::AppNotFound)
        }
    }

    pub fn queue_task<T: 'static + AppMgmtTask>(&self, task: T) {
        self.pool.execute(move || {
            task.run();
        });
    }

    pub fn apply_download(
        &mut self,
        apps_item: &mut AppsItem,
        available_dir: &Path,
        manifest: &Manifest,
        path: &Path,
        is_update: bool,
    ) -> Result<(), AppsServiceError> {
        let manifest_url = apps_item.get_manifest_url();
        let app_name = apps_item.get_name();
        let installed_dir = path.join("installed").join(&app_name);
        let webapp_dir = path.join("vroot").join(&app_name);

        // We can now replace the installed one with new version.
        let _ = remove_dir_all(&installed_dir);
        let _ = remove_file(&webapp_dir);

        if let Err(err) = fs::rename(available_dir, &installed_dir) {
            error!(
                "Rename installed dir failed: {} -> {} : {:?}",
                available_dir.display(),
                installed_dir.display(),
                err
            );
            return Err(AppsServiceError::FilesystemFailure);
        }
        if let Err(err) = symlink(&installed_dir, &webapp_dir) {
            error!("Link installed app failed: {:?}", err);
            return Err(AppsServiceError::FilesystemFailure);
        }

        // Cannot serve a flat file in the same dir as zip.
        // Save the downloaded update manifest file in cached dir.
        let download_update_manifest = installed_dir.join("update.webmanifest");
        if download_update_manifest.exists() {
            let cached_dir = path.join("cached").join(&app_name);
            let cached_update_manifest = cached_dir.join("update.webmanifest");
            let _ = AppsStorage::ensure_dir(&cached_dir);
            if let Err(err) = fs::rename(&download_update_manifest, &cached_update_manifest) {
                error!(
                    "Rename update manifest failed: {} -> {} : {:?}",
                    download_update_manifest.display(),
                    cached_update_manifest.display(),
                    err
                );
                return Err(AppsServiceError::FilesystemFailure);
            }
        } else {
            // Install via cmd do not have update manifest.
            apps_item.set_update_manifest_url(&String::new());
        }

        apps_item.set_install_state(AppsInstallState::Installed);
        if is_update {
            apps_item.set_update_state(AppsUpdateState::Idle);
        }
        let _ = self.save_app(is_update, &apps_item)?;

        // Relay the request to Gecko using the bridge.
        let features = match manifest.get_b2g_features() {
            Some(b2g_features) => {
                JsonValue::from(serde_json::to_value(&b2g_features).unwrap_or(json!(null)))
            }
            _ => JsonValue::from(json!(null)),
        };

        let bridge = GeckoBridgeService::shared_state();
        if is_update {
            bridge.lock().apps_service_on_update(manifest_url, features);
        } else {
            bridge
                .lock()
                .apps_service_on_install(manifest_url, features);
        }

        Ok(())
    }

    pub fn apply_pwa(
        &mut self,
        apps_item: &mut AppsItem,
        download_dir: &Path,
        manifest: &Manifest,
        path: &Path,
        is_update: bool,
    ) -> Result<(), AppsServiceError> {
        let cached_dir = path.join("cached").join(apps_item.get_name());

        // We can now replace the installed one with new version.
        let _ = fs::remove_dir_all(&cached_dir);
        if let Err(err) = fs::rename(&download_dir, &cached_dir) {
            error!(
                "Rename installed dir failed: {} -> {} : {:?}",
                download_dir.display(),
                cached_dir.display(),
                err
            );
            return Err(AppsServiceError::FilesystemFailure);
        }

        apps_item.set_install_state(AppsInstallState::Installed);
        if is_update {
            apps_item.set_update_state(AppsUpdateState::Idle);
        }
        let _ = self
            .register_app(apps_item)
            .map_err(|_| AppsServiceError::RegistrationError)?;

        // Relay the request to Gecko using the bridge.
        if let Some(b2g_features) = manifest.get_b2g_features() {
            let bridge = GeckoBridgeService::shared_state();
            let b2g_features =
                JsonValue::from(serde_json::to_value(&b2g_features).unwrap_or(json!(null)));
            // for pwa app, the permission need to be applied to the host origin
            if is_update {
                bridge
                    .lock()
                    .apps_service_on_update(apps_item.runtime_url(), b2g_features);
            } else {
                bridge
                    .lock()
                    .apps_service_on_install(apps_item.runtime_url(), b2g_features);
            }
        }
        Ok(())
    }

    pub fn get_all(&self) -> Vec<AppsObject> {
        self.apps_list.clone()
    }

    pub fn get_b2g_features(&self, manifest_url: &str, data_path: &str) -> JsonValue {
        let base_dir = Path::new(&data_path);

        if let Some(app) = &self.get_by_manifest_url(manifest_url) {
            let app_dir = app.get_appdir(&base_dir).unwrap_or_default();
            if let Ok(manifest) = AppsStorage::load_manifest(&app_dir) {
                if let Some(b2g_features) = manifest.get_b2g_features() {
                    return JsonValue::from(
                        serde_json::to_value(&b2g_features).unwrap_or(json!(null)),
                    );
                };
            };
        }
        JsonValue::from(json!(null))
    }

    pub fn get_by_manifest_url(&self, manifest_url: &str) -> Option<AppsItem> {
        if let Some(db) = &self.db {
            if let Ok(app) = db.get_by_manifest_url(manifest_url) {
                return Some(app);
            }
        }
        None
    }

    pub fn get_by_update_url(&self, update_url: &str) -> Option<AppsItem> {
        if let Some(db) = &self.db {
            if let Ok(app) = db.get_by_update_url(update_url) {
                debug!("get_by_update_url app {:#?}", app);
                return Some(app);
            }
        }
        None
    }

    pub fn get_first_by_name(&self, name: &str) -> Option<AppsItem> {
        if let Some(db) = &self.db {
            if let Ok(app) = db.get_first_by_name(name) {
                return Some(app);
            }
        }
        None
    }

    pub fn get_vhost_port(&self) -> u16 {
        self.vhost_port
    }

    fn apps_list_add_or_update(&mut self, app: &AppsObject) {
        for item in self.apps_list.iter_mut() {
            if item.manifest_url == app.manifest_url {
                *item = app.clone();
                return;
            }
        }
        self.apps_list.push(app.clone());
    }

    fn register_or_replace(&mut self, app: &AppsItem) -> Result<(), AppsError> {
        if let Some(ref mut db) = &mut self.db {
            db.add(app)?;
            Ok(())
        } else {
            Err(AppsError::NoDb)
        }
    }

    // Register an app in the registry
    // When re-install an existing app, we just overwrite existing one.
    // In future, we could change the policy.
    pub fn register_app(&mut self, apps_item: &AppsItem) -> Result<(), RegistrationError> {
        self.register_or_replace(apps_item)?;
        self.apps_list_add_or_update(&AppsObject::from(apps_item));
        Ok(())
    }

    pub fn validate(manifest_url: &str, manifest: &Manifest) -> Result<(), RegistrationError> {
        if manifest_url.is_empty() {
            return Err(RegistrationError::ManifestUrlMissing);
        }

        if let Err(err) = manifest.is_valid() {
            return Err(RegistrationError::WrongManifest(err));
        }

        Ok(())
    }

    pub fn unregister(&mut self, manifest_url: &str) -> bool {
        if let Some(ref mut db) = &mut self.db {
            let start_count = db.count().unwrap_or(0);
            let _ = db.remove_by_manifest_url(manifest_url);
            let end_count = db.count().unwrap_or(0);
            return start_count != end_count;
        }
        false
    }

    fn unregister_app(&mut self, manifest_url: &str) -> Result<String, RegistrationError> {
        if manifest_url.is_empty() {
            return Err(RegistrationError::ManifestUrlMissing);
        }

        if self.unregister(&manifest_url) {
            self.apps_list
                .retain(|item| item.manifest_url != manifest_url);
            Ok(manifest_url.to_string())
        } else {
            Err(RegistrationError::ManifestURLNotFound)
        }
    }

    pub fn uninstall_app(
        &mut self,
        manifest_url: &str,
        data_path: &str,
    ) -> Result<String, AppsServiceError> {
        let app = match self.get_by_manifest_url(manifest_url) {
            Some(app) => app,
            None => return Err(AppsServiceError::AppNotFound),
        };
        if !app.get_removable() {
            return Err(AppsServiceError::UninstallForbidden);
        }
        if let Ok(manifest_url) = self.unregister_app(manifest_url) {
            if AppsStorage::remove_app(&app, data_path).is_ok() {
                // Relay the request to Gecko using the bridge.
                let bridge = GeckoBridgeService::shared_state();
                bridge.lock().apps_service_on_uninstall(app.runtime_url());
                return Ok(manifest_url);
            }
        }
        error!("Unregister app failed: {}", manifest_url);
        Err(AppsServiceError::UninstallError)
    }

    // Register an app to the registry
    // When re-install an existing app, we just overwrite existing one.
    // In future, we could change the policy.
    pub fn update_app(&mut self, apps_item: &AppsItem) -> Result<(), RegistrationError> {
        // TODO: need to unregister something for update if needed.

        Ok(self.register_app(apps_item)?)
    }

    pub fn count(&self) -> usize {
        if let Some(db) = &self.db {
            db.count().unwrap_or(0) as _
        } else {
            0
        }
    }

    pub fn register_on_boot(&mut self, config: &Config) -> Result<(), AppsError> {
        let current = env::current_dir().unwrap();
        let base_dir = current.join(&config.data_path);

        // In case of app needes to be add or removed after the system update,
        // need to wait the gecko bridge to uninstall the local storage and permissions.
        if let Err(err) = self.check_system_update(config) {
            error!("Check post system update error: {:?}", err);
        }

        if let Some(db) = &self.db {
            let apps = db.get_all()?;
            for app in &apps {
                let app_dir = app.get_appdir(&base_dir).unwrap_or_default();
                if let Ok(manifest) = AppsStorage::load_manifest(&app_dir) {
                    // Relay the request to Gecko using the bridge.
                    let features = match manifest.get_b2g_features() {
                        Some(b2g_features) => JsonValue::from(
                            serde_json::to_value(&b2g_features).unwrap_or(json!(null)),
                        ),
                        _ => JsonValue::from(json!(null)),
                    };

                    debug!("Register on boot manifest_url: {}", &app.get_manifest_url());
                    let bridge = GeckoBridgeService::shared_state();
                    bridge
                        .lock()
                        .apps_service_on_boot(app.runtime_url(), features);
                }
            }
            // Notify the bridge that we processed all apps on boot.
            GeckoBridgeService::shared_state()
                .lock()
                .apps_service_on_boot_done();
        }
        Ok(())
    }

    pub fn check_system_update(&mut self, config: &Config) -> Result<(), AppsError> {
        let current = env::current_dir().unwrap();
        let root_dir = current.join(config.root_path.clone());
        let setting_service = SettingsService::shared_state();
        let build_fingerprint =
            AndroidProperties::get("ro.system.build.fingerprint", "").unwrap_or_default();

        debug!("apps system update: fingerprint is {}", &build_fingerprint);
        match setting_service.lock().db.get("system.saved.fingerprint") {
            Ok(saved_fingerprint) => {
                if *saved_fingerprint.as_str().unwrap_or("").to_string() == build_fingerprint {
                    debug!("apps system update: The fingerprint is the same.");
                    return Ok(());
                }
            }
            Err(err) => {
                debug!(
                    "apps system update: Failed to Get saved fingerprint: {:?}",
                    err
                );
            }
        }

        let webapps_json = root_dir.join("webapps.json");
        if let Ok(mut sys_apps) = AppsStorage::read_webapps(webapps_json) {
            if let Some(ref mut db) = &mut self.db {
                let apps = db.get_all()?;
                for app in &apps {
                    let app_name = app.get_name();
                    debug!("apps system update: check {}", &app_name);
                    if sys_apps
                        .iter()
                        .any(|sys_app| !app.get_preloaded() || sys_app.get_name() == app_name)
                    {
                        if !app.get_preloaded() {
                            continue;
                        }
                        // Case 1: if the preloaded app is newer after system update,
                        let sys_app_dir = root_dir.join(&app_name);
                        if let Ok(manifest) = AppsStorage::load_manifest(&sys_app_dir) {
                            if !is_new_version(&app.get_version(), &manifest.get_version()) {
                                continue;
                            }
                            debug!("apps system update: found newer version of {}", &app_name);
                            let mut new_app = app.clone();
                            if AppsStorage::remove_app(&app, &config.data_path).is_err() {
                                error!("apps system update: Failed to remove old app.");
                            }
                            if let Ok(app) =
                                AppsStorage::add_app(&mut new_app, config, self.vhost_port)
                            {
                                if let Err(err) = db.add(&app) {
                                    error!(
                                        "apps system update: Failed to update new app to db {:?}",
                                        err
                                    );
                                }
                            }
                        };
                        continue;
                    }
                    // Case 2: the preloaded app is removed after system update,
                    // remove it from the db and registry.
                    if config.allow_remove_preloaded {
                        debug!(
                            "apps system update: fremove an old preloaded app {}",
                            &app_name
                        );
                        let _ = db.remove_by_manifest_url(&app.get_manifest_url());
                        match AppsStorage::remove_app(&app, &config.data_path) {
                            Ok(_) => {
                                self.apps_list
                                    .retain(|item| item.manifest_url != app.get_manifest_url());
                                // Relay the request to Gecko using the bridge.
                                let bridge = GeckoBridgeService::shared_state();
                                bridge
                                    .lock()
                                    .apps_service_on_uninstall(app.runtime_origin());
                            }
                            Err(err) => {
                                error!("apps system update: remvoe app failed: {}", err);
                            }
                        }
                    }
                }
                // Case 3: there are new preloaded apps after system update,
                // add it to the db and registry.
                for sys_app in &mut sys_apps {
                    let app_name = sys_app.get_name();
                    if apps.iter().find(|app| app.get_name() == app_name).is_none() {
                        debug!("apps system update: found new preload app {}", &app_name);
                        if let Ok(app) = AppsStorage::add_app(sys_app, config, self.vhost_port) {
                            self.apps_list.push(AppsObject::from(&app));
                            if let Err(err) = db.add(&app) {
                                error!(
                                    "apps system update: Failed to update new app to db {:?}",
                                    err
                                );
                            }
                        }
                    }
                }
            }
        }

        let setting = [SettingInfo {
            name: "system.saved.fingerprint".into(),
            value: Value::String(build_fingerprint).into(),
        }];
        thread::spawn(move || match setting_service.lock().db.set(&setting) {
            Ok(_) => debug!("apps system update: Successfully write fingerprint to setting."),
            Err(err) => error!(
                "apps system update: Failed to write fingerprint to setting: {:?}",
                err
            ),
        });

        Ok(())
    }

    pub fn set_enabled(
        &mut self,
        manifest_url: &str,
        status: AppsStatus,
        data_dir: &Path,
    ) -> Result<(AppsObject, bool), AppsServiceError> {
        if let Some(mut app) = self.get_by_manifest_url(manifest_url) {
            let mut status_changed = false;
            if app.get_status() == status {
                return Ok((AppsObject::from(&app), status_changed));
            }

            status_changed = true;
            if let Some(ref mut db) = &mut self.db {
                let _ = db
                    .update_status(manifest_url, status)
                    .map_err(|_| AppsServiceError::FilesystemFailure)?;

                let app_dir = app.get_appdir(&data_dir).unwrap_or_default();
                let disabled_dir = data_dir.join("disabled");
                let app_disabled_dir = disabled_dir.join(&app.get_name());

                match status {
                    AppsStatus::Disabled => {
                        if !AppsStorage::ensure_dir(&disabled_dir) {
                            return Err(AppsServiceError::FilesystemFailure);
                        }
                        let _ = fs::rename(app_dir, app_disabled_dir)
                            .map_err(|_| AppsServiceError::FilesystemFailure)?;
                    }
                    AppsStatus::Enabled => {
                        let _ = fs::rename(app_disabled_dir, app_dir)
                            .map_err(|_| AppsServiceError::FilesystemFailure)?;
                    }
                }

                app.set_status(status);

                Ok((AppsObject::from(&app), status_changed))
            } else {
                Err(AppsServiceError::InvalidState)
            }
        } else {
            Err(AppsServiceError::AppNotFound)
        }
    }

    pub fn check_need_restart(&self, need_restart: bool) {
        if need_restart {
            let pool = self.pool.clone();
            thread::spawn(move || {
                // Calling join from a thread within the pool will cause a deadlock.
                pool.join();
                ::std::process::exit(0);
            });
        }
    }
}

pub trait AppMgmtTask: Send {
    fn run(&self);
}

fn observe_bridge(shared_data: Shared<AppsSharedData>, config: &Config) {
    let receiver = GeckoBridgeService::shared_state().lock().observe_bridge();
    loop {
        if GeckoBridgeService::shared_state().lock().is_ready() {
            if let Err(err) = shared_data.lock().registry.register_on_boot(config) {
                error!("register_on_boot failed: {}", err);
            }
            let mut shared = shared_data.lock();
            if shared.state != AppsServiceState::Running {
                shared.state = AppsServiceState::Running;
            }
        }
        if let Err(err) = receiver.recv() {
            // In normal case, it shouldn't reach here.
            // Sleep 1 sec to avoid the busy loop if something is wrong.
            error!("receiver error: {:?}", err);
            thread::sleep(Duration::from_secs(1));
        }
    }
}

pub fn start(shared_data: Shared<AppsSharedData>, vhost_port: u16) {
    let config = shared_data.lock().config.clone();
    match AppsRegistry::initialize(&config, vhost_port) {
        Ok(registry) => {
            debug!("Apps registered successfully");
            shared_data.lock().registry = registry;

            // Monitor gecko bridge and register on b2g restart.
            let shared_with_observer = shared_data.clone();
            thread::Builder::new()
                .name("apps_bridge".to_string())
                .spawn(move || observe_bridge(shared_with_observer, &config))
                .expect("Failed to start apps_bridge thread");

            let shared_with_actor = shared_data.clone();
            thread::Builder::new()
                .name("apps_actor".to_string())
                .spawn(move || apps_actor::start_webapp_actor(shared_with_actor))
                .expect("Failed to start apps_actor thread");

            let shared_with_scheduler = shared_data.clone();
            let sender = update_scheduler::start(shared_with_scheduler);
            {
                let mut shared = shared_data.lock();
                shared.scheduler = Some(sender);
            }
        }
        Err(err) => {
            error!("Error initializing apps registry: {}", err);
        }
    }
}

#[test]
fn test_init_apps_from_system() {
    use crate::config;
    use crate::manifest::Icons;
    use config::Config;
    use url::Url;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir", current.display());
    let test_path = Path::new(&test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(&test_path);

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path: root_path.clone(),
        data_path: test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("production"),
        updater_socket: String::from("updater_socket"),
        user_agent: String::from("user_agent"),
        allow_remove_preloaded: true,
    };

    let registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count: {}", registry.count());
    assert_eq!(6, registry.count());

    // Verify the preload test pwa app.
    if let Some(app) =
        registry.get_by_update_url("https://preloadpwa.domain.url/manifest.webmanifest")
    {
        assert_eq!(app.get_name(), "preloadpwa");

        let cached_dir = test_path.join("cached");
        let update_manifest = cached_dir.join(app.get_name()).join("manifest.webmanifest");
        let manifest = Manifest::read_from(&update_manifest).unwrap();

        // start url should be absolute url of remote address
        assert_eq!(
            manifest.get_start_url(),
            "https://preloadpwa.domain.url/index.html"
        );

        // icon url should be relative path of local cached address
        if let Some(icons_value) = manifest.get_icons() {
            let icons: Vec<Icons> = serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
            let manifest_url_base = Url::parse(&app.get_manifest_url()).unwrap().join("/");
            for icon in icons {
                let icon_src = icon.get_src();
                let icon_url = Url::parse(&icon_src).unwrap();
                let icon_url_base = icon_url.join("/");
                let icon_path = format!("{}{}", cached_dir.to_str().unwrap(), icon_url.path());

                assert_eq!(icon_url_base, manifest_url_base);
                assert!(Path::new(&icon_path).is_file(), "Error in icon path.");
            }
        } else {
            panic!();
        }
    } else {
        panic!();
    }

    let test_json = Path::new(&root_path).join("webapps.json");
    if let Ok(test_items) = AppsStorage::read_webapps(&test_json) {
        for item in &test_items {
            assert!(registry.get_first_by_name(&item.get_name()).is_some());
            assert_eq!(
                registry
                    .get_by_manifest_url(&item.get_manifest_url())
                    .unwrap()
                    .get_name(),
                item.get_name()
            );
        }
    } else {
        panic!("Wrong apps config in data.");
    }
}

#[test]
fn test_register_app() {
    use crate::config;
    use crate::manifest::B2GFeatures;
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir-actor", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&test_dir));

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path,
        data_path: test_dir,
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
        user_agent: String::from("user_agent"),
        allow_remove_preloaded: true,
    };

    let vhost_port = 80;
    let mut registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(6, registry.count());

    // Test register_app - invalid name
    let name = "";
    let update_url = "some_url";
    let launch_path = "some_path";
    let b2g_features = None;
    let manifest = Manifest::new(name, launch_path, b2g_features);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), Some(&update_url))
        .err()
        .unwrap();
    assert_eq!(app_name, AppsServiceError::InvalidAppName);

    // Test register_app - without update url
    let name = "some_name";
    let launch_path = "some_path";
    let b2g_features = None;
    let manifest = Manifest::new(name, launch_path, b2g_features);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), None)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    if registry.register_app(&apps_item).is_err() {
        panic!();
    }

    // 1. Normal register install
    // 2. Re-register as update
    // 3. Unregister
    let update_url = "https://store.helloworld.local/manifest.webmanifest";

    // Normal register an app
    let name = "helloworld";
    let launch_path = "/index.html";
    let version = "1.0.0";
    let data = r#"{"version": "1.0.0"}"#;
    let b2g_features: B2GFeatures = serde_json::from_str(data).unwrap();
    let manifest = Manifest::new(name, launch_path, Some(b2g_features));
    let app_name = registry
        .get_unique_name(&manifest.get_name(), Some(update_url))
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item).unwrap();

    // Verify the manifet url is as expeced
    let expected_manifest_url = "http://helloworld.localhost/manifest.webmanifest";
    let manifest_url = apps_item.get_manifest_url();
    assert_eq!(&manifest_url, expected_manifest_url);

    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), name);
            assert_eq!(test.get_version(), version);
        }
        None => panic!("helloworld is not found."),
    }

    // Re-register an app
    let name = "helloworld";
    let launch_path = "/index.html";
    let version = "1.0.1";
    let data = r#"{"version": "1.0.1"}"#;
    let b2g_features: B2GFeatures = serde_json::from_str(data).unwrap();
    let manifest1 = Manifest::new(name, launch_path, Some(b2g_features));
    let app_name = registry
        .get_unique_name(&manifest1.get_name(), Some(&update_url))
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);
    if !manifest1.get_version().is_empty() {
        apps_item.set_version(&manifest1.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item).unwrap();

    let manifest_url = apps_item.get_manifest_url();
    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), app_name);
            assert_eq!(test.get_version(), version);
        }
        None => panic!("helloworld is not found."),
    }

    // Unregister an app
    registry.unregister_app(&manifest_url).unwrap();

    // Sould be 6 apps left
    assert_eq!(7, registry.count());

    let manifest_url = apps_item.get_manifest_url();
    if registry.get_by_manifest_url(&manifest_url).is_some() {
        panic!("helloworld should be unregisterd");
    }
}

#[test]
fn test_unregister_app() {
    use crate::config;
    use crate::manifest::B2GFeatures;
    use config::Config;

    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!(
        "{}/test-fixtures/test-unregister-apps-dir",
        current.display()
    );

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&test_dir));

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path,
        data_path: test_dir,
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
        user_agent: String::from("user_agent"),
        allow_remove_preloaded: true,
    };

    let vhost_port = 8081;
    let mut registry = AppsRegistry::initialize(&config, vhost_port).unwrap();

    // Test unregister_app - invalid manifest url
    let manifest_url = "";
    if let Err(err) = registry.unregister_app(&manifest_url) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::ManifestUrlMissing),
        );
    } else {
        panic!();
    }

    // Test unregister_app - invalid manifest url
    let manifest_url = "https://store.helloworld.local/manifest.webmanifest";
    if let Err(err) = registry.unregister_app(&manifest_url) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::ManifestURLNotFound),
        );
    } else {
        panic!();
    }

    // Normal case
    let name = "helloworld";
    let launch_path = "/index.html";
    let version = "1.0.1";
    let data = r#"{"version": "1.0.1"}"#;
    let b2g_features: B2GFeatures = serde_json::from_str(data).unwrap();
    let manifest1 = Manifest::new(name, launch_path, Some(b2g_features));
    let app_name = registry
        .get_unique_name(&manifest1.get_name(), None)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    if !manifest1.get_version().is_empty() {
        apps_item.set_version(&manifest1.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item).unwrap();

    assert_eq!(7, registry.count());

    // Verify the manifet url is as expeced
    let expected_manfiest_url = "http://helloworld.localhost:8081/manifest.webmanifest";
    let manifest_url = apps_item.get_manifest_url();
    assert_eq!(&manifest_url, expected_manfiest_url);

    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), name);
            assert_eq!(test.get_version(), version);
        }
        None => panic!("helloworld is not found."),
    }

    // Uninstall it
    let manifest_url = apps_item.get_manifest_url();
    registry.unregister_app(&manifest_url).unwrap();

    // 5 apps left
    assert_eq!(6, registry.count());

    let manifest_url = apps_item.get_manifest_url();
    if registry.get_by_manifest_url(&manifest_url).is_some() {
        panic!("helloworld should be unregistered");
    }
}

#[test]
fn test_apply_download() {
    use crate::config;
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir-apply", current.display());
    let test_path = Path::new(&_test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(&test_path);

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
        user_agent: String::from("user_agent"),
        allow_remove_preloaded: true,
    };

    let vhost_port = 80;
    let mut registry = match AppsRegistry::initialize(&config, vhost_port) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(6, registry.count());

    let src_app = current.join("test-fixtures/apps-from/helloworld/application.zip");
    let src_manifest = current.join("test-fixtures/apps-from/helloworld/update.webmanifest");
    let available_dir = test_path.join("downloading/helloworld");
    let available_app = available_dir.join("application.zip");
    let update_manifest = available_dir.join("update.webmanifest");

    // Test 1: new app name new updat_url.
    if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
        println!("{:?}", err);
    }
    let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
    let _ = fs::copy(src_manifest.as_path(), update_manifest.as_path()).unwrap();
    let manifest = validate_package(&available_app.as_path()).unwrap();
    let update_url = "https://test0.helloworld/manifest.webmanifest";

    let app_name = registry
        .get_unique_name(&manifest.get_name(), Some(&update_url))
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);
    let expected_update_manfiest_url =
        format!("http://cached.localhost/{}/update.webmanifest", &app_name);

    if registry
        .apply_download(
            &mut apps_item,
            &available_dir.as_path(),
            &manifest,
            &test_path,
            false,
        )
        .is_ok()
    {
        assert_eq!(app_name, "helloworld");
    } else {
        panic!();
    }

    let app = registry.get_by_update_url(update_url).unwrap();
    assert_eq!(app.get_update_manifest_url(), expected_update_manfiest_url);

    // Test 2: same app name different updat_url 1 - 100.
    let mut count = 1;
    loop {
        if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
            println!("{:?}", err);
        }
        let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
        let _ = fs::copy(src_manifest.as_path(), update_manifest.as_path()).unwrap();
        let manifest = validate_package(&available_app.as_path()).unwrap();
        let update_url = &format!("https://test{}.helloworld/manifest.webmanifest", count);
        let app_name = registry
            .get_unique_name(&manifest.get_name(), Some(&update_url))
            .unwrap();
        let mut apps_item = AppsItem::default(&app_name, vhost_port);
        if !manifest.get_version().is_empty() {
            apps_item.set_version(&manifest.get_version());
        }
        apps_item.set_install_state(AppsInstallState::Installing);
        apps_item.set_update_url(update_url);
        if registry
            .apply_download(
                &mut apps_item,
                &available_dir.as_path(),
                &manifest,
                &test_path,
                false,
            )
            .is_ok()
        {
            let expected_name = format!("helloworld{}", count);
            assert_eq!(app_name, expected_name);
        } else if count <= 999 {
            panic!();
        }
        if count == 100 {
            break;
        }
        count += 1;
    }

    // Test 3: same app name and old updat_url as test 0.
    if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
        println!("{:?}", err);
    }
    let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
    let _ = fs::copy(src_manifest.as_path(), update_manifest.as_path()).unwrap();
    let manifest = validate_package(&available_app.as_path()).unwrap();
    let update_url = "https://test0.helloworld/manifest.webmanifest";

    let app_name = registry
        .get_unique_name(&manifest.get_name(), Some(&update_url))
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);
    if registry
        .apply_download(
            &mut apps_item,
            &available_dir.as_path(),
            &manifest,
            &test_path,
            true,
        )
        .is_ok()
    {
        assert_eq!(app_name, "helloworld");
    } else {
        panic!();
    }

    // Test 4: app name is reserved and not allowed.
    let src_app = current.join("test-fixtures/apps-from/invalid/name_not_allow.zip");
    if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
        println!("{:?}", err);
    }
    let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
    let manifest = validate_package(&available_app.as_path()).unwrap();
    let update_url = "https://invalidname.localhost/manifest.webmanifest";
    let app_name = registry.get_unique_name(&manifest.get_name(), Some(&update_url));
    assert_eq!(app_name, Err(AppsServiceError::InvalidAppName));

    {
        let manifest_url = apps_item.get_manifest_url();

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Enabled);
            }
            None => panic!(),
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path)
        {
            assert_eq!(app.status, AppsStatus::Enabled);
            assert!(!changed);
        } else {
            panic!();
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Disabled, &test_path)
        {
            assert_eq!(app.status, AppsStatus::Disabled);
            assert!(changed);
        } else {
            panic!();
        }

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Disabled);
            }
            None => panic!(),
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path)
        {
            assert_eq!(app.status, AppsStatus::Enabled);
            assert!(changed);
        } else {
            panic!();
        }

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Enabled);
            }
            None => panic!(),
        }
    }
}
