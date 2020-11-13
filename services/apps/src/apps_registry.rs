/// Registry for apps services:
/// - Init app from system to data partition.
/// - Maintain the runtime set of apps.
/// - Exposes high level apis to manipulate the registry.
use crate::apps_actor;
use crate::apps_item::AppsItem;
use crate::apps_storage::{validate_package, AppsStorage};
use crate::apps_utils;
use crate::config::Config;
use crate::downloader::{DownloadError, Downloader};
use crate::generated::common::*;
use crate::manifest::{Icons, Manifest, ManifestError};
use crate::registry_db::RegistryDb;
use crate::shared_state::AppsSharedData;
use crate::update_manifest::UpdateManifest;
use crate::update_scheduler;
use common::traits::DispatcherId;
use common::traits::Shared;
use common::JsonValue;
use hex_slice::AsHex;
use log::{debug, error, info};
use md5::{Digest, Md5};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::convert::From;
use std::env;
use std::fs;
use std::fs::{remove_dir_all, remove_file, File};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use threadpool::ThreadPool;
use url::Url;
use version_compare::{CompOp, Version};
use zip::ZipArchive;
use zip_utils::verify_zip;

// Relay the request to Gecko using the bridge.
use common::traits::Service;
use geckobridge::service::GeckoBridgeService;

pub trait AppMgmtTask: Send {
    fn run(&self);
}

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

fn hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

fn read_zip_manifest<P: AsRef<Path>>(zip_file: P) -> Result<Manifest, AppsError> {
    let file = File::open(zip_file)?;
    let mut archive = ZipArchive::new(file)?;
    let manifest = archive.by_name("manifest.webapp")?;
    let value: Manifest = serde_json::from_reader(manifest)?;
    Ok(value)
}

fn read_webapps<P: AsRef<Path>>(apps_json_file: P) -> Result<Vec<AppsItem>, AppsError> {
    let file = File::open(apps_json_file)?;
    let reader = BufReader::new(file);
    let value: Vec<AppsItem> = serde_json::from_reader(reader)?;
    Ok(value)
}

// Loads the manifest for an app:
// First tries to load it from app_dir/application.zip!manifest.webapp if it's present,
// and fallbacks to app_dir/manifest.webapp
fn load_manifest(app_dir: &PathBuf) -> Result<Manifest, AppsError> {
    let zipfile = app_dir.join("application.zip");
    if let Ok(manifest) = read_zip_manifest(zipfile.as_path()) {
        Ok(manifest)
    } else {
        let manifest = app_dir.join("manifest.webapp");
        let file = File::open(manifest)?;
        let reader = BufReader::new(file);
        let value: Manifest = serde_json::from_reader(reader)?;
        Ok(value)
    }
}

// A struct that removes a directory's content when it is dropped, unless
// it is marked explicitely to not do so.
struct DirRemover {
    path: PathBuf,
    remove: bool,
}

impl DirRemover {
    fn new(path: &PathBuf) -> Self {
        Self {
            path: path.clone(),
            remove: true,
        }
    }

    fn keep(&mut self) {
        self.remove = false;
    }
}

impl Drop for DirRemover {
    fn drop(&mut self) {
        if self.remove {
            let _ = remove_dir_all(&self.path);
        }
    }
}

#[derive(Default)]
pub struct AppsRegistry {
    pub downloader: Downloader, // Keeping the downloader around for reuse.
    pool: ThreadPool,           // The thread pool used to run tasks.
    db: Option<RegistryDb>,     // The sqlite DB wrapper.
    cert_type: String,          // Root CA to be trusted to verify the signature.
    vhost_port: u16,            // Keeping vhost vhost_port number in registry
    pub event_broadcaster: AppsEngineEventBroadcaster,
}

impl AppsRegistry {
    pub fn add_dispatcher(&mut self, dispatcher: &AppsEngineEventDispatcher) -> DispatcherId {
        self.event_broadcaster.add(dispatcher)
    }

    pub fn remove_dispatcher(&mut self, id: DispatcherId) {
        self.event_broadcaster.remove(id)
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

        // Open the registry database.
        let mut db = RegistryDb::new(db_dir.join(&"apps.sqlite"))?;
        let count = db.count()?;

        // No apps yet, load the default ones from the json file.
        if count == 0 {
            info!("Initializing apps registry from {}", config.root_path);

            if !data_dir.exists() {
                fs::create_dir(&data_dir)?;
            }

            let sys_apps = root_dir.join("webapps.json");
            match read_webapps(sys_apps) {
                Ok(mut apps) => {
                    for app in &mut apps {
                        let app_name = app.get_name();
                        let source = root_dir.join(&app_name);
                        let dest = data_dir.join(&app_name);
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
        }

        Ok(Self {
            downloader: Downloader::default(),
            pool: ThreadPool::new(3),
            db: Some(db),
            cert_type: config.cert_type.clone(),
            vhost_port,
            event_broadcaster: AppsEngineEventBroadcaster::default(),
        })
    }

    fn get_available_zip(&mut self, url: &str, path: &Path) -> Result<PathBuf, AppsMgmtError> {
        let zip_path = path.join("application.zip");
        debug!("Dowloading {} to {}", url, zip_path.as_path().display());
        if let Err(err) = self.downloader.download(url, zip_path.as_path()) {
            error!(
                "Downloading {} to {} failed: {:?}",
                url,
                zip_path.as_path().display(),
                err
            );
            return Err(AppsMgmtError::PackageDownloadFailed);
        }
        Ok(zip_path)
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

    fn get_update_manifest(
        &mut self,
        url: &str,
        path: &Path,
    ) -> Result<(PathBuf, PathBuf), AppsMgmtError> {
        let base_path = path.join("downloading");
        let available_dir = AppsStorage::get_app_dir(&base_path, &hash(url).to_string())?;

        let update_manifest = available_dir.join("update.manifest");
        debug!("dowload {} to {}", url, available_dir.display());
        if let Err(err) = self.downloader.download(url, update_manifest.as_path()) {
            error!(
                "Downloading {} to {} failed: {:?}",
                url,
                update_manifest.as_path().display(),
                err
            );
            if format!("{:?}", DownloadError::Http("304".into())) == format!("{:?}", err) {
                return Err(AppsMgmtError::DownloadNotMoidified);
            }
            return Err(AppsMgmtError::ManifestDownloadFailed);
        }
        Ok((available_dir, update_manifest))
    }

    fn save_app(
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

    pub fn download_and_apply(
        &mut self,
        webapp_path: &str,
        update_url: &str,
        is_update: bool,
    ) -> Result<AppsObject, AppsServiceError> {
        let path = Path::new(&webapp_path);
        let (available_dir, update_manifest) = self
            .get_update_manifest(update_url, &path)
            .map_err(|_| AppsServiceError::DownloadManifestFailed)?;
        let manifest = Manifest::read_from(update_manifest.clone())
            .map_err(|_| AppsServiceError::InvalidManifest)?;

        let app_name = self.get_unique_name(&manifest.get_name(), &update_url)?;
        // Need create appsItem object and add to db to reflect status
        let mut apps_item = AppsItem::default(&app_name, self.get_vhost_port());
        if !manifest.get_version().is_empty() {
            apps_item.set_version(&manifest.get_version());
        }
        apps_item.set_update_url(update_url);
        apps_item.set_install_state(AppsInstallState::Installing);
        let _ = self.save_app(is_update, &apps_item, &manifest)?;

        self.event_broadcaster
            .broadcast_app_installing(AppsObject::from(&apps_item));
        let mut available_dir_remover = DirRemover::new(&available_dir);

        let available_dir = available_dir.as_path();
        let update_manifest = update_manifest.as_path();
        info!("update_manifest: {}", update_manifest.display());

        let update_manifest = match UpdateManifest::read_from(update_manifest) {
            Ok(manifest) => manifest,
            Err(_) => {
                let _ = self.unregister(&apps_item.get_manifest_url());
                return Err(AppsServiceError::InvalidManifest);
            }
        };

        if update_manifest.package_path.is_empty() {
            error!("No package path.");
            let _ = self.unregister(&apps_item.get_manifest_url());
            return Err(AppsServiceError::InvalidManifest);
        }

        if AppsStorage::available_disk_space(&webapp_path) < update_manifest.packaged_size * 2 {
            error!("Do not have enough disk space.");
            let _ = self.unregister(&apps_item.get_manifest_url());
            return Err(AppsServiceError::DiskSpaceNotEnough);
        }

        let available_zip =
            match self.get_available_zip(&update_manifest.package_path, available_dir) {
                Ok(package) => package,
                Err(_) => {
                    apps_item.set_install_state(AppsInstallState::Pending);
                    let _ = self.save_app(is_update, &apps_item, &manifest);
                    return Err(AppsServiceError::DownloadPackageFailed);
                }
            };

        info!("available_zip: {}", available_zip.display());
        if let Err(err) = verify_zip(available_zip.as_path(), &self.cert_type) {
            error!("Verify zip error: {:?}", err);
            apps_item.set_install_state(AppsInstallState::Pending);
            let _ = self.save_app(is_update, &apps_item, &manifest);
            return Err(AppsServiceError::InvalidSignature);
        }

        let manifest = match validate_package(available_zip.as_path()) {
            Ok(manifest) => manifest,
            Err(_) => {
                apps_item.set_install_state(AppsInstallState::Pending);
                let _ = self.save_app(is_update, &apps_item, &manifest);
                return Err(AppsServiceError::InvalidPackage);
            }
        };

        if !apps_utils::compare_manifests(&update_manifest, &manifest) {
            apps_item.set_install_state(AppsInstallState::Pending);
            let _ = self.save_app(is_update, &apps_item, &manifest);
            return Err(AppsServiceError::InvalidManifest);
        }

        // Here we can emit ready to apply download if we have sparate steps
        // asking user to apply download
        if let Err(err) =
            self.apply_download(&mut apps_item, &available_dir, &manifest, &path, is_update)
        {
            apps_item.set_install_state(AppsInstallState::Pending);
            let _ = self.save_app(is_update, &apps_item, &manifest);
            return Err(err);
        };

        // Everything went fine, don't remove the available_dir directory.
        available_dir_remover.keep();
        Ok(AppsObject::from(&apps_item))
    }

    pub fn check_for_update(
        &mut self,
        webapp_path: &str,
        update_url: &str,
        is_auto_update: bool,
    ) -> Result<Option<AppsObject>, AppsServiceError> {
        debug!("check_for_update is_auto_update {}", is_auto_update);
        let app = match self.get_by_update_url(update_url) {
            Some(app) => app,
            None => {
                return Err(AppsServiceError::AppNotFound);
            }
        };

        let path = Path::new(&webapp_path);
        let (_, update_manifest) = match self.get_update_manifest(update_url, &path) {
            Ok((available_dir, update_manifest)) => (available_dir, update_manifest),
            Err(err) => {
                if format!("{:?}", AppsMgmtError::DownloadNotMoidified) == format!("{:?}", err) {
                    return Ok(None);
                }
                return Err(AppsServiceError::DownloadManifestFailed);
            }
        };

        if compare_version_hash(&app, update_manifest) {
            let mut app_obj = AppsObject::from(&app);
            app_obj.allowed_auto_download = is_auto_update;

            Ok(Some(app_obj))
        } else {
            Ok(None)
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
        let webapp_dir = path.join(&app_name);

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

    pub fn download_and_apply_pwa(
        &mut self,
        webapp_path: &str,
        update_url: &str,
    ) -> Result<AppsObject, AppsServiceError> {
        let path = Path::new(&webapp_path);
        let download_dir =
            AppsStorage::get_app_dir(&path.join("downloading"), &hash(update_url).to_string())
                .map_err(|_| AppsServiceError::DownloadManifestFailed)?;
        let download_manifest = download_dir.join("manifest.webmanifest");
        let downloader = Downloader::default();

        // 1. download manfiest to cache dir.
        debug!("dowload {} to {}", update_url, download_manifest.display());
        if let Err(err) = downloader.download(update_url, download_manifest.as_path()) {
            error!(
                "Downloading {} to {} failed: {:?}",
                update_url,
                download_manifest.as_path().display(),
                err
            );
            return Err(AppsServiceError::DownloadManifestFailed);
        }
        let mut manifest = Manifest::read_from(&download_manifest)
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        let app_name = self.get_unique_name(&manifest.get_name(), &update_url)?;
        let mut apps_item = AppsItem::default_pwa(&app_name, self.get_vhost_port());
        apps_item.set_install_state(AppsInstallState::Installing);
        apps_item.set_update_url(&update_url);
        self.event_broadcaster
            .broadcast_app_installing(AppsObject::from(&apps_item));

        // 2-1. download icons to cached dir.
        let update_url_base =
            Url::parse(update_url).map_err(|_| AppsServiceError::InvalidManifest)?;
        let manifest_url_base = Url::parse(&apps_item.get_manifest_url())
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        if let Some(icons_value) = manifest.get_icons() {
            let mut icons: Vec<Icons> =
                serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
            for icon in &mut icons {
                let mut icon_src = icon.get_src();
                // If this is an absolute uri remove the leading / when building the download path
                // so we don't end up trying to use a /some/invalid/path/icon.png path.
                if icon_src.starts_with('/') {
                    let _ = icon_src.remove(0);
                }
                let icon_path = download_dir.join(&icon_src);
                let icon_dir = icon_path.parent().unwrap();
                let icon_url = update_url_base
                    .join(&icon.get_src())
                    .map_err(|_| AppsServiceError::InvalidManifest)?;
                let _ = AppsStorage::ensure_dir(&icon_dir);
                if let Err(err) = downloader.download(icon_url.as_str(), icon_path.as_path()) {
                    error!(
                        "Failed to download icon {} -> {:?} : {:?}",
                        icon_url, icon_path, err
                    );
                }
                let icon_cached_url = manifest_url_base
                    .join(&icon_src)
                    .map_err(|_| AppsServiceError::InvalidManifest)?;
                icon.set_src(icon_cached_url.as_str());
            }
            manifest.set_icons(serde_json::to_value(icons).unwrap());
        }

        // 2-2. update start url in cached manifest to absolute url
        let start_url = update_url_base
            .join(&manifest.get_start_url())
            .map_err(|_| AppsServiceError::InvalidStartUrl)?;
        manifest.set_start_url(start_url.as_str());
        Manifest::write_to(&download_manifest, &manifest)
            .map_err(|_| AppsServiceError::InvalidManifest)?;

        // 3. finish installation and reigster the pwa app.
        self.apply_pwa(&mut apps_item, &download_dir.as_path(), &manifest, &path)?;

        Ok(AppsObject::from(&apps_item))
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

    pub fn register_or_replace(&mut self, app: &AppsItem) -> Result<(), AppsError> {
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
                let webapp_dir = path.join(app_name);

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
        let data_dir = current.join(&config.data_path);

        if let Some(db) = &self.db {
            let apps = db.get_all()?;
            for app in &apps {
                let app_dir = data_dir.join(&app.get_name());
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
                    let app_data_dir = data_dir.join(&app.get_name());
                    let _ = remove_file(&app_data_dir);
                } else if status == AppsStatus::Enabled {
                    let installed_dir = data_dir.join("installed").join(&app.get_name());
                    let app_data_dir = data_dir.join(&app.get_name());
                    let app_root_dir = root_dir.join(&app.get_name());
                    if installed_dir.exists() {
                        let _ = symlink(&installed_dir, &app_data_dir);
                    } else if app_root_dir.exists() {
                        let _ = symlink(&app_root_dir, &app_data_dir);
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

fn compute_manifest_hash<P: AsRef<Path>>(p: P) -> Result<String, AppsError> {
    let mut file = File::open(p)?;
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content);
    let mut hasher = Md5::new();
    hasher.update(content);
    let result = hasher.finalize();

    Ok(format!("{:02x}", result.plain_hex(false)))
}

fn compare_version_hash<P: AsRef<Path>>(app: &AppsItem, update_manifest: P) -> bool {
    let manifest = match UpdateManifest::read_from(&update_manifest) {
        Ok(manifest) => manifest,
        Err(_) => return false,
    };

    debug!("from update manifest{:#?}", manifest);
    debug!("hash from registry {}", app.get_manifest_hash());

    let mut is_update_available = false;
    if app.get_version().is_empty() || manifest.version.is_empty() {
        let hash_str = match compute_manifest_hash(&update_manifest) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        debug!("hash from update manifest{}", hash_str);
        if hash_str != app.get_manifest_hash() {
            is_update_available = true;
        }
    }

    if !app.get_version().is_empty() && !manifest.version.is_empty() {
        if let Some(manifest_version) = Version::from(&manifest.version) {
            if let Some(app_version) = Version::from(&app.get_version()) {
                is_update_available = manifest_version.compare(&app_version) == CompOp::Gt;
            }
        }
    }

    debug!("compare_version_hash update {}", is_update_available);
    is_update_available
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
            thread::spawn(move || observe_bridge(shared_with_observer, &config));

            let shared_with_actor = shared_data.clone();
            thread::spawn(move || apps_actor::start_webapp_actor(shared_with_actor));

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
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&_test_dir));

    if let Err(err) = fs::create_dir_all(PathBuf::from(_test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("production"),
    };

    let registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count: {}", registry.count());
    assert_eq!(4, registry.count());

    let test_json = Path::new(&_root_dir).join("webapps.json");
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
        assert!(false, "Wrong apps config in data.");
    }
}

#[test]
fn test_register_app() {
    use crate::config;
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir-actor", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&_test_dir));

    if let Err(err) = fs::create_dir_all(PathBuf::from(_test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
    };

    let mut registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(4, registry.count());

    let vhost_port = registry.get_vhost_port();

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
        assert!(false);
    }

    // 1. Normal register install
    // 2. Re-register as update
    // 3. Unregister
    let update_url = format!("https://store.helloworld.local/manifest.webapp");

    // Normal register an app
    let name = "helloworld";
    let launch_path = "/index.html";
    let version = "1.0.0";
    let manifest = Manifest::new(name, launch_path, version);
    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    } else {
        assert!(false, "No version found.");
    }

    registry.register_app(&apps_item, &manifest).unwrap();

    // Verify the manifet url is as expeced
    let expected_manifest_url = format!("http://helloworld.localhost/manifest.webapp");
    let manifest_url = apps_item.get_manifest_url();
    assert_eq!(manifest_url, expected_manifest_url);

    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), name);
            assert_eq!(test.get_version(), version);
        }
        None => assert!(false, "helloworld is not found."),
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
        assert!(false, "No version found.");
    }

    registry.register_app(&apps_item, &manifest1).unwrap();

    let manifest_url = apps_item.get_manifest_url();
    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), app_name);
            assert_eq!(test.get_version(), version);
        }
        None => assert!(false, "helloworld is not found."),
    }

    // Unregister an app
    registry.unregister_app(&update_url).unwrap();

    // Sould be 4 apps left
    assert_eq!(4, registry.count());

    let manifest_url = apps_item.get_manifest_url();
    match registry.get_by_manifest_url(&manifest_url) {
        Some(_) => assert!(false, "helloworld should be unregisterd"),
        None => {}
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
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!(
        "{}/test-fixtures/test-unregister-apps-dir",
        current.display()
    );

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(Path::new(&_test_dir));

    if let Err(err) = fs::create_dir_all(PathBuf::from(_test_dir.clone())) {
        println!("{:?}", err);
    }

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
    };

    let mut registry = AppsRegistry::initialize(&config, 8081).unwrap();

    // Test unregister_app - invalid updater url
    let update_url = "";
    let vhost_port = registry.get_vhost_port();
    if let Err(err) = registry.unregister_app(&update_url) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::UpdateUrlMissing),
        );
    } else {
        assert!(false);
    }

    // Test unregister_app - invalid updater url
    let update_url = format!("https://store.helloworld.local/manifest.webapp");
    if let Err(err) = registry.unregister_app(&update_url) {
        assert_eq!(
            format!("{}", err),
            format!("{}", RegistrationError::ManifestURLNotFound),
        );
    } else {
        assert!(false);
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
        assert!(false, "No version found.");
    }

    registry.register_app(&apps_item, &manifest1).unwrap();

    assert_eq!(5, registry.count());

    // Verify the manifet url is as expeced
    let expected_manfiest_url = format!("http://helloworld.localhost:8081/manifest.webapp");
    let manifest_url = apps_item.get_manifest_url();
    assert_eq!(manifest_url, expected_manfiest_url);

    match registry.get_by_manifest_url(&manifest_url) {
        Some(test) => {
            assert_eq!(test.get_name(), name);
            assert_eq!(test.get_version(), version);
        }
        None => assert!(false, "helloworld is not found."),
    }

    // Uninstall it
    let update_url = apps_item.get_update_url();
    registry.unregister_app(&update_url).unwrap();

    // 4 apps left
    assert_eq!(4, registry.count());

    let manifest_url = apps_item.get_manifest_url();
    match registry.get_by_manifest_url(&manifest_url) {
        Some(_) => assert!(false, "helloworld should be unregisterd"),
        None => {}
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
    };

    let mut registry = match AppsRegistry::initialize(&config, 80) {
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
    let update_url = "https://test0.helloworld/manifest.webapp";

    let vhost_port = registry.get_vhost_port();
    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);

    if let Ok(_) = registry.apply_download(
        &mut apps_item,
        &available_dir.as_path(),
        &manifest,
        &test_path,
        false,
    ) {
        assert_eq!(app_name, "helloworld");
    } else {
        assert!(false);
    }

    // Test 2: same app name different updat_url 1 - 100.
    let mut count = 1;
    loop {
        if let Err(err) = fs::create_dir_all(available_dir.as_path()) {
            println!("{:?}", err);
        }
        let _ = fs::copy(src_app.as_path(), available_app.as_path()).unwrap();
        let manifest = validate_package(&available_app.as_path()).unwrap();
        let update_url = &format!("https://test{}.helloworld/manifest.webapp", count);
        let app_name = registry
            .get_unique_name(&manifest.get_name(), &update_url)
            .unwrap();
        let mut apps_item = AppsItem::default(&app_name, vhost_port);
        if !manifest.get_version().is_empty() {
            apps_item.set_version(&manifest.get_version());
        }
        apps_item.set_install_state(AppsInstallState::Installing);
        apps_item.set_update_url(update_url);
        if let Ok(_) = registry.apply_download(
            &mut apps_item,
            &available_dir.as_path(),
            &manifest,
            &test_path,
            false,
        ) {
            let expected_name = format!("helloworld{}", count);
            assert_eq!(app_name, expected_name);
        } else {
            if count <= 999 {
                assert!(false);
            }
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
    let update_url = "https://test0.helloworld/manifest.webapp";

    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .unwrap();
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(update_url);
    if let Ok(_) = registry.apply_download(
        &mut apps_item,
        &available_dir.as_path(),
        &manifest,
        &test_path,
        true,
    ) {
        assert_eq!(app_name, "helloworld");
    } else {
        assert!(false);
    }

    {
        let manifest_url = apps_item.get_manifest_url();

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Enabled);
            }
            None => assert!(false),
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path, &root_path)
        {
            assert_eq!(app.status, AppsStatus::Enabled);
            assert!(!changed);
        } else {
            assert!(false);
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Disabled, &test_path, &root_path)
        {
            assert_eq!(app.status, AppsStatus::Disabled);
            assert!(changed);
        } else {
            assert!(false);
        }

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Disabled);
            }
            None => assert!(false),
        }

        if let Ok((app, changed)) =
            registry.set_enabled(&manifest_url, AppsStatus::Enabled, &test_path, &root_path)
        {
            assert_eq!(app.status, AppsStatus::Enabled);
            assert!(changed);
        } else {
            assert!(false);
        }

        match registry.get_by_manifest_url(&manifest_url) {
            Some(app) => {
                assert_eq!(app.get_status(), AppsStatus::Enabled);
            }
            None => assert!(false),
        }
    }
}

#[test]
fn test_compare_version_hash() {
    let _ = env_logger::try_init();

    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/compare-version-hash/sample_update_manifest.webapp",
        current.display()
    );

    // version compare
    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item.set_version("1.0.0");

        assert!(compare_version_hash(&apps_item, &manifest_path));
    }

    // version compare
    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item.set_version("1.0.1");

        assert!(!compare_version_hash(&apps_item, &manifest_path));
    }

    // version compare
    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item.set_version("1.0.2");

        assert!(!compare_version_hash(&apps_item, &manifest_path));
    }

    // hash
    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item.set_manifest_hash("6bfc26d201fd94xxxxbdd31b63f4aa54");

        assert!(compare_version_hash(&apps_item, &manifest_path));
    }

    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item.set_manifest_hash("6bfc26d201fd94a431bdd31b63f4aa54");

        assert!(!compare_version_hash(&apps_item, &manifest_path));
    }
}

#[test]
fn test_apply_pwa() {
    use crate::config;
    use config::Config;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir-pwa.
    let current = env::current_dir().unwrap();
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir-pwa", current.display());
    let test_path = Path::new(&_test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = fs::remove_dir_all(&test_path);

    println!("Register from: {}", &_root_dir);
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
    };

    let mut registry = match AppsRegistry::initialize(&config, 80) {
        Ok(v) => v,
        Err(err) => {
            panic!("err: {:?}", err);
        }
    };

    println!("registry.count(): {}", registry.count());
    assert_eq!(4, registry.count());

    // Test 1: apply from a local dir
    let src_manifest = current.join("test-fixtures/apps-from/pwa/manifest.webmanifest");
    let update_url = "https://pwa1.test/manifest.webmanifest";
    let download_dir = AppsStorage::get_app_dir(
        &test_path.join("downloading"),
        &format!("{}", hash(update_url)),
    )
    .unwrap();
    let download_manifest = download_dir.join("manifest.webmanifest");

    if let Err(err) = fs::create_dir_all(download_dir.as_path()) {
        println!("{:?}", err);
    }
    let _ = fs::copy(&src_manifest, &download_manifest).unwrap();
    let manifest = Manifest::read_from(&download_manifest).unwrap();
    let app_name = registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .unwrap();
    if let Some(icons_value) = manifest.get_icons() {
        let icons: Vec<Icons> = serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
        assert_eq!(4, icons.len());
    } else {
        assert!(false);
    }
    let mut apps_item = AppsItem::default(&app_name, registry.get_vhost_port());
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(&update_url);

    match registry.apply_pwa(
        &mut apps_item,
        &download_dir.as_path(),
        &manifest,
        &test_path,
    ) {
        Ok(_) => {
            assert_eq!(apps_item.get_name(), "helloworld");
            assert_eq!(apps_item.get_install_state(), AppsInstallState::Installed);
        }
        Err(err) => {
            println!("err: {:?}", err);
            assert!(false);
        }
    }

    // Test 2: download and apply from a remote url
    let app_url = "https://testpwa.github.io/manifest.webmanifest";
    match registry.download_and_apply_pwa(&_test_dir, app_url) {
        Ok(app) => {
            assert_eq!(app.name, "hellopwa");
        }
        Err(err) => {
            println!("err: {:?}", err);
            assert!(false);
        }
    }
    if let Some(app) = registry.get_by_update_url(app_url) {
        assert_eq!(app.get_name(), "hellopwa");

        let cached_dir = test_path.join("cached").join(app.get_name());
        let update_manifest = cached_dir.join("manifest.webmanifest");
        let manifest = Manifest::read_from(&update_manifest).unwrap();

        // start url should be absolute url of remote address
        assert_eq!(
            manifest.get_start_url(),
            "https://testpwa.github.io/index.html"
        );

        // icon url should be relative path of local cached address
        if let Some(icons_value) = manifest.get_icons() {
            let icons: Vec<Icons> = serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
            assert_eq!(4, icons.len());
            let manifest_url_base = Url::parse(&app.get_manifest_url()).unwrap().join("/");
            for icon in icons {
                let icon_src = icon.get_src();
                let icon_url_base = Url::parse(&icon_src).unwrap().join("/");
                assert_eq!(icon_url_base, manifest_url_base);
            }
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}
