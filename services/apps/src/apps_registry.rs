/// Registry for apps services:
/// - Init app from system to data partition.
/// - Maintain the runtime set of apps.
/// - Exposes high level apis to manipulate the registry.
use crate::apps_actor;
use crate::apps_item::AppsItem;
use crate::apps_storage::AppsStorage;
use crate::config::Config;
use crate::generated::common::*;
use crate::manifest::{Manifest, ManifestError};
use crate::registry_db::RegistryDb;
use crate::shared_state::AppsSharedData;
use crate::update_scheduler;
use common::traits::{DispatcherId, Shared};
use common::JsonValue;
use log::{debug, error, info};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::convert::From;
use std::env;
use std::fs;
use std::fs::{remove_dir_all, remove_file, File};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufReader;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use threadpool::ThreadPool;
use zip::ZipArchive;

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
    #[error("AppsMgmtError")]
    ManifestDownloadFailed,
    #[error("AppsMgmtError")]
    ManifestReadFailed,
    #[error("AppsMgmtError")]
    ManifestInvalid,
    #[error("AppsMgmtError")]
    DownloadFailed,
    #[error("AppsMgmtError")]
    DownloadNotMoidified,
    #[error("AppsMgmtError")]
    PackageCorrupt,
    #[error("AppsMgmtError")]
    PackageDownloadFailed,
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
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

fn read_zip_manifest<P: AsRef<Path>>(
    zip_file: P,
    manifest_name: &str,
) -> Result<Manifest, AppsError> {
    let file = File::open(zip_file)?;
    let mut archive = ZipArchive::new(file)?;
    let manifest = archive.by_name(manifest_name)?;
    let value: Manifest = serde_json::from_reader(manifest)?;
    Ok(value)
}

fn read_webapps<P: AsRef<Path>>(apps_json_file: P) -> Result<Vec<AppsItem>, AppsError> {
    let file = File::open(apps_json_file)?;
    let reader = BufReader::new(file);
    let value: Vec<AppsItem> = serde_json::from_reader(reader)?;
    Ok(value)
}

// Loads the manifest for an app from app_dir/application.zip!manifest.webmanifest
// and fallbacks to app_dir/manifest.webmanifest if needed.
fn load_manifest(app_dir: &PathBuf) -> Result<Manifest, AppsError> {
    let zipfile = app_dir.join("application.zip");
    if let Ok(manifest) = read_zip_manifest(zipfile.as_path(), "manifest.webmanifest") {
        Ok(manifest)
    } else if let Ok(manifest) = read_zip_manifest(zipfile.as_path(), "manifest.webapp") {
        Ok(manifest)
    } else {
        let manifest = app_dir.join("manifest.webmanifest");
        let file = File::open(manifest)?;
        let reader = BufReader::new(file);
        let value: Manifest = serde_json::from_reader(reader)?;
        Ok(value)
    }
}

#[derive(Default)]
pub struct AppsRegistry {
    pool: ThreadPool,       // The thread pool used to run tasks.
    db: Option<RegistryDb>, // The sqlite DB wrapper.
    // cert_type: String,          // Root CA to be trusted to verify the signature.
    vhost_port: u16, // Keeping vhost vhost_port number in registry
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

            let sys_apps = root_dir.join("webapps.json");
            match read_webapps(sys_apps) {
                Ok(mut apps) => {
                    for app in &mut apps {
                        let app_name = app.get_name();
                        let source = root_dir.join(&app_name);
                        let dest = vroot_dir.join(&app_name);
                        app.set_manifest_url(&AppsItem::new_manifest_url(&app_name, vhost_port));
                        if let Err(err) = symlink(&source, &dest) {
                            // Don't fail if the symlink already exists.
                            if err.kind() != std::io::ErrorKind::AlreadyExists {
                                error!(
                                    "Failed to create symlink {:?} -> {:?} : {}",
                                    source, dest, err
                                );
                                return Err(err.into());
                            }
                        }
                        // Add the app to the database.
                        db.add(&app)?;
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

        Ok(Self {
            pool: ThreadPool::new(3),
            db: Some(db),
            vhost_port,
            event_broadcaster: AppsEngineEventBroadcaster::default(),
        })
    }

    // Returns a sanitized version of the name, usable as:
    // - a filename
    // - a subdomain
    // The result is ASCII only with no whitespace.
    pub fn sanitize_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
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
        update_url: &str,
    ) -> Result<String, AppsServiceError> {
        if let Some(db) = &self.db {
            // The update_url is unique return name from the db.
            if let Ok(app) = db.get_by_update_url(update_url) {
                return Ok(app.get_name());
            }

            let mut unique_name: String = Self::sanitize_name(name);
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
                        .find(|app| app.get_name() == unique_name)
                        .is_none()
                    {
                        break;
                    }
                    if count > 999 {
                        return Err(AppsServiceError::InvalidAppName);
                    }
                    unique_name = format!("{}{}", name, count);
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
        manifest: &Manifest,
    ) -> Result<(), AppsServiceError> {
        if is_update {
            self.update_app(apps_item, manifest)
                .map_err(|_| AppsServiceError::RegistrationError)?
        } else {
            self.register_app(apps_item, manifest)
                .map_err(|_| AppsServiceError::RegistrationError)?
        }

        Ok(())
    }

    pub fn clear(
        &mut self,
        manifest_url: &str,
        data_type: ClearType,
    ) -> Result<(), AppsServiceError> {
        let type_str = match data_type {
            ClearType::Browser => "Browser",
            ClearType::Storage => "Storage",
        };

        Ok(GeckoBridgeService::shared_state()
            .lock()
            .apps_service_on_clear(manifest_url.to_string(), type_str.to_string())
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

        apps_item.set_install_state(AppsInstallState::Installed);
        if is_update {
            apps_item.set_update_state(AppsUpdateState::Idle);
        }
        let _ = self.save_app(is_update, &apps_item, &manifest);

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
        let _ = self
            .register_app(apps_item, manifest)
            .map_err(|_| AppsServiceError::RegistrationError)?;

        // Relay the request to Gecko using the bridge.
        if let Some(b2g_features) = manifest.get_b2g_features() {
            let bridge = GeckoBridgeService::shared_state();
            let b2g_features =
                JsonValue::from(serde_json::to_value(&b2g_features).unwrap_or(json!(null)));
            // for pwa app, the permission need to be applied to the host origin
            bridge
                .lock()
                .apps_service_on_install(apps_item.get_update_url(), b2g_features);
        }
        Ok(())
    }

    pub fn get_all(&self) -> Vec<AppsObject> {
        if let Some(db) = &self.db {
            if let Ok(apps) = db.get_all() {
                return apps.iter().map(AppsObject::from).collect();
            }
        }
        vec![]
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
    pub fn register_app(
        &mut self,
        apps_item: &AppsItem,
        manifest: &Manifest,
    ) -> Result<(), RegistrationError> {
        let _ = AppsRegistry::validate(
            &apps_item.get_manifest_url(),
            &apps_item.get_update_url(),
            manifest,
        )?;

        self.register_or_replace(apps_item)?;
        Ok(())
    }

    fn validate(
        manifest_url: &str,
        update_url: &str,
        manifest: &Manifest,
    ) -> Result<(), RegistrationError> {
        if manifest_url.is_empty() {
            return Err(RegistrationError::ManifestUrlMissing);
        }
        if update_url.is_empty() {
            return Err(RegistrationError::UpdateUrlMissing);
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

    pub fn unregister_app(&mut self, update_url: &str) -> Result<String, RegistrationError> {
        if update_url.is_empty() {
            return Err(RegistrationError::UpdateUrlMissing);
        }

        if let Some(app) = self.get_by_update_url(update_url) {
            let manifest_url = app.get_manifest_url();
            if self.unregister(&manifest_url) {
                // Relay the request to Gecko using the bridge.
                let bridge = GeckoBridgeService::shared_state();
                bridge
                    .lock()
                    .apps_service_on_uninstall(manifest_url.clone());
                Ok(manifest_url)
            } else {
                Err(RegistrationError::ManifestURLNotFound)
            }
        } else {
            Err(RegistrationError::ManifestURLNotFound)
        }
    }

    pub fn uninstall_app(
        &mut self,
        app_name: &str,
        update_url: &str,
        data_path: &str,
    ) -> Result<String, AppsServiceError> {
        match self.unregister_app(update_url) {
            Ok(manifest_url) => {
                let path = Path::new(&data_path);
                let installed_dir = path.join("installed").join(app_name);
                let webapp_dir = path.join("vroot").join(app_name);

                let _ = remove_file(&webapp_dir);
                let _ = remove_dir_all(&installed_dir);

                Ok(manifest_url)
            }
            Err(err) => {
                error!("Unregister app failed: {:?}", err);
                Err(AppsServiceError::AppNotFound)
            }
        }
    }

    // Register an app to the registry
    // When re-install an existing app, we just overwrite existing one.
    // In future, we could change the policy.
    pub fn update_app(
        &mut self,
        apps_item: &AppsItem,
        manifest: &Manifest,
    ) -> Result<(), RegistrationError> {
        // TODO: need to unregister something for update if needed.

        Ok(self.register_app(apps_item, manifest)?)
    }

    pub fn count(&self) -> usize {
        if let Some(db) = &self.db {
            db.count().unwrap_or(0) as _
        } else {
            0
        }
    }

    pub fn register_on_boot(&self, config: &Config) -> Result<(), AppsError> {
        let current = env::current_dir().unwrap();
        let vroot_dir = current.join(&config.data_path).join("vroot");

        if let Some(db) = &self.db {
            let apps = db.get_all()?;
            for app in &apps {
                let app_dir = vroot_dir.join(&app.get_name());
                let manifest = load_manifest(&app_dir)?;

                // Relay the request to Gecko using the bridge.
                let features = match manifest.get_b2g_features() {
                    Some(b2g_features) => {
                        JsonValue::from(serde_json::to_value(&b2g_features).unwrap_or(json!(null)))
                    }
                    _ => JsonValue::from(json!(null)),
                };

                debug!("Register on boot manifest_url: {}", &app.get_manifest_url());
                let bridge = GeckoBridgeService::shared_state();
                bridge
                    .lock()
                    .apps_service_on_boot(app.get_manifest_url(), features);
            }
            // Notify the bridge that we processed all apps on boot.
            GeckoBridgeService::shared_state()
                .lock()
                .apps_service_on_boot_done();
        }
        Ok(())
    }

    pub fn set_enabled(
        &mut self,
        manifest_url: &str,
        status: AppsStatus,
        data_dir: &Path,
        root_dir: &Path,
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

                if status == AppsStatus::Disabled {
                    let app_vroot_dir = data_dir.join("vroot").join(&app.get_name());
                    let _ = remove_file(&app_vroot_dir);
                } else if status == AppsStatus::Enabled {
                    let installed_dir = data_dir.join("installed").join(&app.get_name());
                    let app_vroot_dir = data_dir.join("vroot").join(&app.get_name());
                    let app_root_dir = root_dir.join(&app.get_name());
                    if installed_dir.exists() {
                        let _ = symlink(&installed_dir, &app_vroot_dir);
                    } else if app_root_dir.exists() {
                        let _ = symlink(&app_root_dir, &app_vroot_dir);
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
            {
                let mut shared = shared_data.lock();
                shared.registry = registry;
                shared.state = AppsServiceState::Running;
            }

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
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&test_dir));

    if let Err(err) = fs::create_dir_all(PathBuf::from(test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path: root_path.clone(),
        data_path: test_dir,
        uds_path: String::from("uds_path"),
        cert_type: String::from("production"),
        updater_socket: String::from("updater_socket"),
    };

    let registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count: {}", registry.count());
    assert_eq!(4, registry.count());

    let test_json = Path::new(&root_path).join("webapps.json");
    if let Ok(test_items) = read_webapps(&test_json) {
        for item in &test_items {
            assert!(registry.get_first_by_name(&item.get_name()).is_some());
            assert_eq!(
                registry
                    .get_by_manifest_url(&item.get_manifest_url())
                    .unwrap()
                    .get_name(),
                item.get_name()
            );
            assert_eq!(
                registry
                    .get_by_update_url(&item.get_update_url())
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
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir-actor", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&test_dir));

    if let Err(err) = fs::create_dir_all(PathBuf::from(test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path,
        data_path: test_dir,
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
    };

    let vhost_port = 80;
    let mut registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(4, registry.count());

    // Test register_app - invalid name
    let name = "";
    let update_url = "some_url";
    let launch_path = "some_path";
    let version = "some_version";
    let manifest = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .err()
        .unwrap();
    assert_eq!(app_name, AppsServiceError::InvalidAppName);

    // Test register_app - invalid update url
    let name = "some_name";
    let update_url = "";
    let launch_path = "some_path";
    let version = "some_version";
    let manifest1 = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);
    if let Err(err) = registry.register_app(&apps_item, &manifest1) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::UpdateUrlMissing)
        );
    } else {
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
    let manifest = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item, &manifest).unwrap();

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
    let manifest1 = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest1.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);
    if !manifest1.get_version().is_empty() {
        apps_item.set_version(&manifest1.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item, &manifest1).unwrap();

    let manifest_url = apps_item.get_manifest_url();
    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), app_name);
            assert_eq!(test.get_version(), version);
        }
        None => panic!("helloworld is not found."),
    }

    // Unregister an app
    registry.unregister_app(&update_url).unwrap();

    // Sould be 4 apps left
    assert_eq!(4, registry.count());

    let manifest_url = apps_item.get_manifest_url();
    if registry.get_by_manifest_url(&manifest_url).is_some() {
        panic!("helloworld should be unregisterd");
    }
}

#[test]
fn test_unregister_app() {
    use crate::config;
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

    if let Err(err) = fs::create_dir_all(PathBuf::from(test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &root_path);
    let config = Config {
        root_path,
        data_path: test_dir,
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
    };

    let vhost_port = 8081;
    let mut registry = AppsRegistry::initialize(&config, vhost_port).unwrap();

    // Test unregister_app - invalid updater url
    let update_url = "";
    if let Err(err) = registry.unregister_app(&update_url) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::UpdateUrlMissing),
        );
    } else {
        panic!();
    }

    // Test unregister_app - invalid updater url
    let update_url = "https://store.helloworld.local/manifest.webmanifest";
    if let Err(err) = registry.unregister_app(update_url) {
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
    let manifest1 = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest1.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);
    if !manifest1.get_version().is_empty() {
        apps_item.set_version(&manifest1.get_version());
    } else {
        panic!("No version found.");
    }

    registry.register_app(&apps_item, &manifest1).unwrap();

    assert_eq!(5, registry.count());

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
    let update_url = apps_item.get_update_url();
    registry.unregister_app(&update_url).unwrap();

    // 4 apps left
    assert_eq!(4, registry.count());

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
    let root_path = Path::new(&_root_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(&test_path);

    if let Err(err) = fs::create_dir_all(&test_path) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
        updater_socket: String::from("updater_socket"),
    };

    let vhost_port = 80;
    let mut registry = match AppsRegistry::initialize(&config, vhost_port) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(4, registry.count());

    let src_app = current.join("test-fixtures/apps-from/helloworld/application.zip");
    let available_dir = test_path.join("downloading/helloworld");
    let available_app = available_dir.join("application.zip");

    // Test 1: new app name new updat_url.
    if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
        println!("{:?}", err);
    }
    let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
    let manifest = validate_package(&available_app.as_path()).unwrap();
    let update_url = "https://test0.helloworld/manifest.webmanifest";

    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
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
        assert_eq!(app_name, "helloworld");
    } else {
        panic!();
    }

    // Test 2: same app name different updat_url 1 - 100.
    let mut count = 1;
    loop {
        if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
            println!("{:?}", err);
        }
        let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
        let manifest = validate_package(&available_app.as_path()).unwrap();
        let update_url = &format!("https://test{}.helloworld/manifest.webmanifest", count);
        let app_name = registry
            .get_unique_name(&manifest.get_name(), &update_url)
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
    let manifest = validate_package(&available_app.as_path()).unwrap();
    let update_url = "https://test0.helloworld/manifest.webmanifest";

    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
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
    let app_name = registry.get_unique_name(&manifest.get_name(), &update_url);
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
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path, &root_path)
        {
            assert_eq!(app.status, AppsStatus::Enabled);
            assert!(!changed);
        } else {
            panic!();
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Disabled, &test_path, &root_path)
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
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path, &root_path)
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
