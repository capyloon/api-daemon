// Helper methods to manage app storage area.

use crate::apps_item::AppsItem;
use crate::apps_registry::{AppsError, AppsMgmtError};
use crate::manifest::{LegacyManifest, Manifest, ManifestError};
use common::log_warning;
use log::{debug, error, warn};
use nix::sys::statvfs;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs::{self, remove_dir_all, File, OpenOptions};
use std::io::{BufReader, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;
use zip::result::ZipError;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Zip package corrupted")]
    WrongZipFormat,
    #[error("Zip package error, {0}")]
    FromZipError(#[from] ZipError),
    #[error("Io error, {0}")]
    IoError(#[from] std::io::Error),
    #[error("Zip package not found")]
    ZipPackageNotFound,
    #[error("Package Manifest Error, {0}")]
    WrongManifest(ManifestError),
}

#[cfg(target_os = "android")]
static APP_LOG_FILE: &str = "/data/local/tmp/app-services.log";
#[cfg(not(target_os = "android"))]
static APP_LOG_FILE: &str = "/tmp/app-services.log";

pub struct AppsStorage;

impl AppsStorage {
    // Read the manifest file from application.zip.
    // In
    //   zip_file: the path of application.zip.
    //   manifest_name: the filename of manifest file.
    // Out
    //   A result of the manifest object or error
    fn read_zip_manifest<T, P: AsRef<Path>>(
        zip_file: P,
        manifest_name: &str,
    ) -> Result<Manifest, PackageError>
    where
        T: for<'de> Deserialize<'de>,
        Manifest: From<T>,
    {
        let file = File::open(zip_file)?;
        let mut archive = ZipArchive::new(file)?;
        let manifest = archive.by_name(manifest_name)?;
        let manifest: T = serde_json::from_reader(manifest)
            .map_err(|err| PackageError::WrongManifest(ManifestError::Json(err)))?;

        Ok(manifest.into())
    }

    // Read the apps item list from a webapp json file.
    // In
    //   apps_json: the path of webapp json file.
    // Out
    //   A result of the array of app items or error
    pub fn read_webapps<P: AsRef<Path>>(apps_json_file: P) -> Result<Vec<AppsItem>, AppsError> {
        let file = File::open(apps_json_file)?;
        let reader = BufReader::new(file);
        let value: Vec<AppsItem> = serde_json::from_reader(reader)?;
        Ok(value)
    }

    // Loads the manifest for an app from app_dir/application.zip!manifest.webmanifest
    // and fallbacks to app_dir/manifest.webmanifest if needed.
    pub fn load_manifest(app_dir: &Path) -> Result<Manifest, AppsError> {
        let zipfile = app_dir.join("application.zip");
        if let Ok(manifest) =
            AppsStorage::read_zip_manifest::<Manifest, _>(&zipfile, "manifest.webmanifest")
        {
            Ok(manifest)
        } else if let Ok(manifest) =
            AppsStorage::read_zip_manifest::<LegacyManifest, _>(&zipfile, "manifest.webapp")
        {
            Ok(manifest)
        } else {
            let manifest = app_dir.join("manifest.webmanifest");
            let file = File::open(manifest)?;
            let reader = BufReader::new(file);
            let value: Manifest = serde_json::from_reader(reader)
                .map_err(|err| AppsError::WrongManifest(ManifestError::Json(err)))?;
            Ok(value)
        }
    }

    // Add an app from the system dir to the data dir.
    // In
    //   app: the app item object to be added.
    //   config: the config of apps service.
    //   vhost_port: the port number used by vhost.
    // Out
    //   A result of the app item or error
    pub fn add_system_dir_app(
        app: &mut AppsItem,
        root_path: &Path,
        data_path: &Path,
        vhost_port: u16,
    ) -> Result<AppsItem, AppsError> {
        let app_name = app.get_name();
        let source = root_path.join(&app_name);
        // The manifest URLs of package apps and PWA apps are different.
        //   Package app: https://[app-name].localhost/manifest.webmanifest
        //   PWA app: https://cached.localhost/[app-name]/manifest.webmanifest
        // Extract the preload PWA app assets to the related dir.
        if app.is_pwa() {
            app.set_manifest_url(AppsItem::new_pwa_url(&app_name, vhost_port));
            let dest = data_path.join("cached").join(&app_name);
            let zip = source.join("application.zip");
            let file = match File::open(&zip) {
                Ok(file) => file,
                Err(err) => {
                    error!("Failed to open {}: {}", &zip.display(), err);
                    return Err(err.into());
                }
            };
            let mut archive = match ZipArchive::new(file) {
                Ok(archive) => archive,
                Err(err) => {
                    error!("Failed to read {}: {}", &zip.display(), err);
                    return Err(err.into());
                }
            };
            if let Err(err) = archive.extract(&dest) {
                error!("Failed to extract {:?} to {:?} : {}", source, dest, err);
                return Err(err.into());
            }
        } else {
            app.set_manifest_url(AppsItem::new_manifest_url(&app_name, vhost_port));
            let dest = data_path.join("vroot").join(&app_name);
            Self::safe_symlink(&source, &dest)?;
        }
        app.set_preloaded(true);
        // Get version from manifest for preloaded apps.
        let app_dir = app.get_appdir(data_path).unwrap_or_default();
        if let Ok(manifest) = AppsStorage::load_manifest(&app_dir) {
            if !manifest.get_version().is_empty() {
                app.set_version(&manifest.get_version());
            }
            if let Some(b2g_features) = manifest.get_b2g_features() {
                if let Some(deeplinks) = b2g_features.get_deeplinks() {
                    if let Ok(config_url) = Url::parse(&deeplinks.config()) {
                        let config_path = source.join("deeplinks_config");
                        match deeplinks.process(&config_url, &config_path, None) {
                            Ok(paths) => app.set_deeplink_paths(Some(paths)),
                            Err(err) => error!("Failed to process deeplink: {:?}", err),
                        }
                    }
                }
                if b2g_features.is_from_legacy() {
                    app.set_legacy_manifest_url();
                    if let Err(err) = Self::write_localized_manifest(&app_dir) {
                        error!("Failed to update localized manifest: {:?}", err);
                    }
                }
            }
        }
        // Return the app to be added to the database.
        Ok(app.clone())
    }

    // Remove the app app files from data dir.
    // In
    //   app: the app item object to be added.
    //   data_path: the root dir of webapp in data.
    // Out
    //   A result of ()  or error
    pub fn remove_app(app: &AppsItem, data_path: &Path) -> Result<(), AppsError> {
        let installed_dir = data_path.join("installed").join(app.get_name());
        let webapp_dir = app.get_appdir(data_path).unwrap_or_default();

        let _ = remove_dir_all(webapp_dir);
        let _ = remove_dir_all(installed_dir);

        Ok(())
    }

    // Ensure the installed directory exists.
    pub fn ensure_dir(dir: &Path) -> bool {
        if !dir.exists() {
            return fs::create_dir_all(dir).is_ok();
        }
        true
    }

    // Returns the available disk space, or 0 if an error occurs.
    pub fn available_disk_space(path: &Path) -> u64 {
        if let Ok(stat) = statvfs::statvfs(path) {
            debug!(
                "vstatsfs for {} : bsize={} bfree={} bavail={}",
                path.display(),
                stat.block_size(),
                stat.blocks_free(),
                stat.blocks_available()
            );
            #[allow(clippy::useless_conversion)]
            return (stat.block_size() * stat.blocks_available()).into();
        }
        0
    }

    // Make sure the directory exists and empty.
    pub fn exist_or_mkdir(path: &Path) -> Result<(), AppsMgmtError> {
        debug!("check and create path: {}", path.display());
        let _ = fs::remove_dir_all(path);
        fs::create_dir_all(path)?;
        Ok(())
    }

    // Ensure and returns the requested path for apps storage.
    pub fn get_app_dir(path: &Path, id: &str) -> Result<PathBuf, AppsMgmtError> {
        let mut app_dir = PathBuf::from(path);
        app_dir.push(id);
        AppsStorage::exist_or_mkdir(&app_dir)?;
        Ok(app_dir)
    }

    // Create a symbolic link and sync to the destination.
    // If the destination does exist, return Ok.
    // In
    //   source: the source path
    //   dest: the destination path
    // Out
    //   A result of success or error
    pub fn safe_symlink(source: &Path, dest: &Path) -> Result<(), AppsError> {
        if let Err(err) = symlink(source, dest) {
            // Don't fail if the symlink already exists.
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                error!(
                    "Failed to create symlink {:?} -> {:?} : {}",
                    source, dest, err
                );
                return Err(err.into());
            }
        }
        let f = File::open(dest)?;
        f.sync_all()?;
        Ok(())
    }

    pub fn log_warn(msg: &str) {
        log_warning!(APP_LOG_FILE, 0, 30, msg);
    }

    pub fn read_warnings() -> String {
        let logfile = Path::new(APP_LOG_FILE);
        fs::read_to_string(logfile).unwrap_or_else(|_| "".into())
    }

    // Write localized manifest file to application zip for legacy app.
    // In
    //   app_dir: the application dir.
    // Out
    //   Result Ok or Error.
    pub fn write_localized_manifest(app_dir: &Path) -> Result<(), PackageError> {
        let app_zip = app_dir.join("application.zip");
        let zip_file = File::open(&app_zip)?;
        let mut archive = ZipArchive::new(zip_file)?;
        let manifest = archive.by_name("manifest.webapp")?;
        let manifest: Value = serde_json::from_reader(manifest)
            .map_err(|err| PackageError::WrongManifest(ManifestError::Json(err)))?;

        if let Some(Value::Object(locales)) = manifest.get("locales") {
            let file = OpenOptions::new()
                .create(false)
                .write(true)
                .read(true)
                .open(app_zip)?;
            let mut zip = ZipWriter::new_append(file)?;
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            let mut localized_manifest = manifest.clone();
            // Do not include the full locales in the localized manifest.
            if let Some(v) = localized_manifest.get_mut("locales") {
                *v = json!({});
            }
            for (locale_key, locale_value) in locales.iter() {
                if let Value::Object(lang) = locale_value {
                    let localized_file = format!("manifest.{}.webapp", locale_key);
                    if archive.by_name(&localized_file).is_ok() {
                        continue;
                    }

                    for (lang_key, lang_value) in lang.iter() {
                        if let Some(v) = localized_manifest.get_mut(lang_key) {
                            *v = lang_value.clone();
                        }
                    }
                    zip.start_file(&localized_file, options)?;
                    match serde_json::to_vec(&localized_manifest) {
                        Ok(value) => {
                            if let Err(err) = zip.write_all(&value) {
                                error!("Faile to write {}, error: {:?}", localized_file, err);
                            }
                        }
                        Err(err) => error!("Manifest {}, error: {:?}", localized_file, err),
                    }
                }
            }
            zip.finish()?;
        }

        Ok(())
    }
}

// Validate application.zip at path.
// Return Manifest for later use
pub fn validate_package<P: AsRef<Path>>(path: P) -> Result<Manifest, PackageError> {
    let manifest =
        match AppsStorage::read_zip_manifest::<Manifest, _>(&path, "manifest.webmanifest") {
            Ok(manifest) => manifest,
            Err(PackageError::WrongManifest(detail)) => {
                return Err(PackageError::WrongManifest(detail))
            }
            Err(_) => {
                AppsStorage::read_zip_manifest::<LegacyManifest, _>(&path, "manifest.webapp")?
            }
        };

    if let Err(err) = manifest.check_validity() {
        error!("validate_package WrongManifest error: {:?}", err);
        return Err(PackageError::WrongManifest(err));
    }

    Ok(manifest)
}

#[test]
fn test_read_legacy_manifest() {
    use std::env;

    let current = env::current_dir().unwrap();
    // Test reading legacy manifest.
    let app_zip = format!(
        "{}/test-fixtures/apps-from/legacy/application.zip",
        current.display()
    );

    match AppsStorage::read_zip_manifest::<LegacyManifest, _>(&app_zip, "manifest.webapp") {
        Ok(manifest) => {
            assert_eq!(manifest.get_name(), "LegacyApp");
            if let Some(b2g_features) = manifest.get_b2g_features() {
                assert!(b2g_features.get_default_locale() == "en-US");
                assert!(b2g_features.get_locales().is_some());
                assert!(b2g_features.get_developer().is_some());
                assert!(b2g_features.get_activities().is_some());
                assert!(b2g_features.get_messages().is_some());
                assert!(b2g_features.get_permissions().is_some());
                assert!(b2g_features.get_version().is_some());
                assert!(b2g_features.get_dependencies().is_some());
            } else {
                panic!("Failed to get b2g_features");
            }
        }
        Err(err) => {
            panic!("Failed to read {} error: {:?}", app_zip, err);
        }
    }
}
