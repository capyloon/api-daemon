// Helper methods to manage app storage area.

use crate::apps_item::AppsItem;
use crate::apps_registry::{AppsError, AppsMgmtError};
use crate::config::Config;
use crate::manifest::{Manifest, ManifestError};
use log::{debug, error};
use nix::sys::statvfs;
use std::env;
use std::fs::{self, remove_dir_all, File};
use std::io::BufReader;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::result::ZipError;
use zip::ZipArchive;

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

pub struct AppsStorage;

impl AppsStorage {
    // Read the manifest file from application.zip.
    // In
    //   zip_file: the path of application.zip.
    //   manifest_name: the filename of manifest file.
    // Out
    //   A result of the manifest object or error
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
    pub fn load_manifest(app_dir: &PathBuf) -> Result<Manifest, AppsError> {
        let zipfile = app_dir.join("application.zip");
        if let Ok(manifest) =
            AppsStorage::read_zip_manifest(zipfile.as_path(), "manifest.webmanifest")
        {
            Ok(manifest)
        } else if let Ok(manifest) =
            AppsStorage::read_zip_manifest(zipfile.as_path(), "manifest.webapp")
        {
            Ok(manifest)
        } else {
            let manifest = app_dir.join("manifest.webmanifest");
            let file = File::open(manifest)?;
            let reader = BufReader::new(file);
            let value: Manifest = serde_json::from_reader(reader)?;
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
    pub fn add_app(
        app: &mut AppsItem,
        config: &Config,
        vhost_port: u16,
    ) -> Result<AppsItem, AppsError> {
        let current = env::current_dir().unwrap();
        let root_dir = current.join(config.root_path.clone());
        let data_dir = current.join(config.data_path.clone());
        let app_name = app.get_name();
        let source = root_dir.join(&app_name);
        // The manifest URLs of package apps and PWA apps are different.
        //   Package app: https://[app-name].localhost/manifest.webmanifest
        //   PWA app: https://cached.localhost/[app-name]/manifest.webmanifest
        // Extract the preload PWA app assets to the related dir.
        if app.is_pwa() {
            app.set_manifest_url(&AppsItem::new_pwa_url(&app_name, vhost_port));
            let dest = data_dir.join("cached").join(&app_name);
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
            app.set_manifest_url(&AppsItem::new_manifest_url(&app_name, vhost_port));
            let dest = data_dir.join("vroot").join(&app_name);
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
        }
        app.set_preloaded(true);
        // Get version from manifest for preloaded apps.
        let app_dir = app.get_appdir(&data_dir).unwrap_or_default();
        if let Ok(manifest) = AppsStorage::load_manifest(&app_dir) {
            if !manifest.get_version().is_empty() {
                app.set_version(&manifest.get_version());
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
    pub fn remove_app(app: &AppsItem, data_path: &str) -> Result<(), AppsError> {
        let path = Path::new(data_path);
        let installed_dir = path.join("installed").join(&app.get_name());
        let webapp_dir = app.get_appdir(&path).unwrap_or_default();

        let _ = remove_dir_all(&webapp_dir);
        let _ = remove_dir_all(&installed_dir);

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
    pub fn available_disk_space(path: &str) -> u64 {
        if let Ok(stat) = statvfs::statvfs(path) {
            debug!(
                "vstatsfs for {} : bsize={} bfree={} bavail={}",
                path,
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
        AppsStorage::exist_or_mkdir(app_dir.as_path())?;
        Ok(app_dir)
    }
}

// Validate application.zip at path.
// Return Manifest for later use
pub fn validate_package_with_name<P: AsRef<Path>>(
    path: P,
    manifest_name: &str,
) -> Result<Manifest, PackageError> {
    let package = File::open(path)?;
    let mut archive = ZipArchive::new(package)?;
    let manifest = archive.by_name(manifest_name)?;

    let manifest: Manifest = match serde_json::from_reader(manifest) {
        Ok(manifest) => manifest,
        Err(err) => {
            error!("validate_package WrongManifest json error: {:?}", err);
            return Err(PackageError::WrongManifest(ManifestError::Json(err)));
        }
    };

    Ok(manifest)
}

// Validate application.zip at path.
// Return Manifest for later use
pub fn validate_package<P: AsRef<Path>>(path: P) -> Result<Manifest, PackageError> {
    let manifest = match validate_package_with_name(&path, "manifest.webmanifest") {
        Ok(manifest) => manifest,
        Err(PackageError::WrongManifest(detail)) => {
            return Err(PackageError::WrongManifest(detail))
        }
        Err(_) => validate_package_with_name(&path, "manifest.webapp")?,
    };

    if let Err(err) = manifest.is_valid() {
        error!("validate_package WrongManifest error: {:?}", err);
        return Err(PackageError::WrongManifest(err));
    }

    Ok(manifest)
}
