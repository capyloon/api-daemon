// Helper methods to manage app storage area.

use crate::apps_registry::AppsMgmtError;
use crate::manifest::{Manifest, ManifestError};
use log::{debug, error};
use nix::sys::statvfs;
use std::fs::{self, File};
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
pub fn validate_package<P: AsRef<Path>>(path: P) -> Result<Manifest, PackageError> {
    let package = File::open(path)?;
    let mut archive = ZipArchive::new(package)?;
    let manifest = archive.by_name("manifest.webapp")?;

    let manifest: Manifest = match serde_json::from_reader(manifest) {
        Ok(manifest) => manifest,
        Err(err) => {
            error!("validate_package WrongManifest json error: {:?}", err);
            return Err(PackageError::WrongManifest(ManifestError::Json(err)));
        }
    };

    if let Err(err) = manifest.is_valid() {
        error!("validate_package WrongManifest error: {:?}", err);
        return Err(PackageError::WrongManifest(err));
    }

    Ok(manifest)
}
