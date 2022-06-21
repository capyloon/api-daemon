use crate::apps_item::AppsItem;
use crate::apps_registry::{hash, AppsError, AppsMgmtError};
use crate::apps_storage::{validate_package, AppsStorage};
use crate::apps_utils;
use crate::downloader::{DownloadError, Downloader, DownloaderInfo};
use crate::generated::common::*;
use crate::manifest::{Icon, Manifest};
use crate::shared_state::AppsSharedData;
use crate::shared_state::DownloadingCanceller;
use crate::update_manifest::UpdateManifest;
use blake2::{Blake2s256, Digest};
use common::traits::Shared;
use log::{debug, error, info};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::convert::From;
use std::fs::{self, remove_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;
use url::Host::Domain;
use url::Url;
use version_compare::{CompOp, Version};
use zip_utils::verify_zip;

struct UpdateManifestResult {
    available_dir: PathBuf,
    update_manifest: PathBuf,
    manifest_etag: Option<String>,
}

// A struct that removes a directory's content when it is dropped, unless
// it is marked explicitely to not do so.
struct DirRemover {
    path: PathBuf,
    remove: bool,
}

impl DirRemover {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
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

struct SenderRemover {
    shared_data: Shared<AppsSharedData>,
    app_update_url: Url,
    delay_delete: bool,
}

impl Drop for SenderRemover {
    fn drop(&mut self) {
        if !self.delay_delete {
            remove_canceller(self.shared_data.clone(), &self.app_update_url);
        }
    }
}

impl SenderRemover {
    pub fn new(shared_data: Shared<AppsSharedData>, app_update_url: &Url) -> Self {
        Self {
            shared_data,
            app_update_url: app_update_url.clone(),
            delay_delete: false,
        }
    }

    pub fn delay_delete(&mut self) {
        self.delay_delete = true;
    }
}

impl Drop for AppsRequest {
    fn drop(&mut self) {
        debug!("AppsRequest drop");
        if let Some(apps_item) = self.installing_apps_item.clone() {
            if let Some(url) = apps_item.get_update_url() {
                remove_canceller(self.shared_data.clone(), &url);
            }
        }
    }
}

// A simpler struct to restore apps status when error happens
// during download_and_apply
struct AppsStatusRestorer {
    need_restore: bool,
    shared_data: Shared<AppsSharedData>,
    is_update: bool,
    apps_item: AppsItem,
}

impl AppsStatusRestorer {
    fn new(shared_data: Shared<AppsSharedData>, is_update: bool, apps_item: AppsItem) -> Self {
        Self {
            need_restore: true,
            shared_data,
            is_update,
            apps_item,
        }
    }

    fn dont_restore(&mut self) {
        self.need_restore = false;
    }
}

impl Drop for AppsStatusRestorer {
    fn drop(&mut self) {
        if self.need_restore {
            self.shared_data
                .lock()
                .registry
                .restore_apps_status(self.is_update, &self.apps_item);
        }
    }
}

pub struct AppsRequest {
    pub downloader: Downloader, // Keeping the downloader around for reuse.
    pub shared_data: Shared<AppsSharedData>,
    installing_apps_item: Option<AppsItem>, // Keep this field to report download failed
}

const DOWNLOAD_TIMEOUT: u64 = 600; // 10 mins

impl AppsRequest {
    pub fn new(shared_data: Shared<AppsSharedData>) -> Result<Self, AppsMgmtError> {
        let (user_agent, lang) = {
            let lock = &shared_data.lock();
            (lock.registry.get_user_agent(), lock.registry.get_lang())
        };
        let downloader =
            Downloader::new(&user_agent, &lang).map_err(|_| AppsMgmtError::DownloaderError)?;
        Ok(Self {
            downloader,
            shared_data,
            installing_apps_item: None,
        })
    }

    // Create app_update_url to cancel_sender map in shared data
    // Cancel the downloading process is available by calling cancel_sender.send()
    // before the downloading finishes.
    fn get_available_zip(
        &mut self,
        app_update_url: &Url,
        url: &Url,
        is_update: bool,
        path: &Path,
    ) -> Result<PathBuf, AppsMgmtError> {
        let zip_path = path.join("application.zip");
        debug!("Dowloading {} to {}", url, zip_path.display());

        let headers = HeaderMap::new();

        let (result_recv, cancel_sender) =
            self.downloader
                .clone()
                .download(url, &zip_path, Some(headers));
        self.set_or_update_canceller(app_update_url, cancel_sender.clone());
        let _sender_remover = SenderRemover::new(self.shared_data.clone(), app_update_url);
        // Check if cancel is called
        if self.check_cancelled(app_update_url) {
            debug!("Download cancelled.");
            let _ = cancel_sender.send(());
            return Err(AppsMgmtError::PackageDownloadFailed(
                DownloadError::Canceled,
            ));
        }

        loop {
            // Wait for 10 mins
            if let Ok(rec) = result_recv.recv_timeout(Duration::from_secs(DOWNLOAD_TIMEOUT)) {
                match rec {
                    Ok(info) => match info {
                        DownloaderInfo::Done => {
                            break;
                        }
                        DownloaderInfo::Progress(progress) => {
                            // We receive progress event
                            debug!("Downloading progress {:?}", info);
                            if let Some(apps_item) = &self.installing_apps_item {
                                let mut app = AppsObject::from(apps_item);
                                app.progress = progress.into();
                                self.shared_data
                                    .lock()
                                    .registry
                                    .broadcast_installing(is_update, app);
                            }
                        }
                        _ => {}
                    },
                    Err(err) => {
                        error!(
                            "Downloading {} to {} failed: {:?}",
                            url,
                            zip_path.display(),
                            err
                        );
                        return Err(AppsMgmtError::PackageDownloadFailed(err));
                    }
                }
            } else {
                return Err(AppsMgmtError::PackageDownloadFailed(DownloadError::Other(
                    "Timed Out".into(),
                )));
            }
        }

        // Check if cancel is called. If not, we remove canceller.
        // Any further cancel requests will be rejected.
        if self.check_cancelled(app_update_url) {
            debug!("Download cancelled after finished.");
            return Err(AppsMgmtError::PackageDownloadFailed(
                DownloadError::Canceled,
            ));
        }

        Ok(zip_path)
    }

    fn get_update_manifest(
        &mut self,
        url: &Url,
        path: &Path,
        some_headers: Option<HeaderMap>,
        canceller_needed: bool, // Check for update does not support cancel
    ) -> Result<UpdateManifestResult, AppsMgmtError> {
        let base_path = path.join("downloading");
        let available_dir = AppsStorage::get_app_dir(&base_path, &hash(url).to_string())?;

        let update_manifest = available_dir.join("update.webmanifest");
        debug!("dowload {} to {}", url, available_dir.display());
        let mut headers = HeaderMap::new();
        if let Some(hders) = some_headers {
            headers = hders;
        }

        let (result_recv, cancel_sender) =
            self.downloader
                .clone()
                .download(url, &update_manifest, Some(headers));
        let sender_remover = if canceller_needed {
            self.set_or_update_canceller(url, cancel_sender);
            Some(SenderRemover::new(self.shared_data.clone(), url))
        } else {
            None
        };

        let mut etag = String::new();
        loop {
            if let Ok(rec) = result_recv.recv_timeout(Duration::from_secs(DOWNLOAD_TIMEOUT)) {
                match rec {
                    Ok(result) => match result {
                        DownloaderInfo::Done => {
                            // Let request drop to remove sender
                            if let Some(mut remover) = sender_remover {
                                remover.delay_delete();
                            };
                            return Ok(UpdateManifestResult {
                                available_dir,
                                update_manifest,
                                manifest_etag: Some(etag),
                            });
                        }
                        DownloaderInfo::Etag(tag) => {
                            etag = tag;
                        }
                        _ => {}
                    },
                    Err(err) => {
                        error!(
                            "Downloading {} to {} failed: {:?}",
                            url,
                            update_manifest.display(),
                            err
                        );
                        if err == DownloadError::Http("304".into()) {
                            return Err(AppsMgmtError::DownloadNotModified);
                        }

                        return Err(AppsMgmtError::ManifestDownloadFailed(err));
                    }
                }
            } else {
                return Err(AppsMgmtError::ManifestDownloadFailed(DownloadError::Other(
                    "Timed Out".into(),
                )));
            }
        }
    }

    pub fn check_for_update(
        &mut self,
        webapp_path: &Path,
        mut app: AppsItem,
        is_auto_update: bool,
    ) -> Result<Option<AppsObject>, AppsServiceError> {
        debug!("check_for_update is_auto_update {}", is_auto_update);
        let mut headers = HeaderMap::new();
        get_etag_header(&app, &mut headers);
        let update_url = app
            .get_update_url()
            .ok_or(AppsServiceError::InvalidUpdateUrl)?;
        let update_manifest_result =
            match self.get_update_manifest(&update_url, webapp_path, Some(headers), false) {
                Ok(update_manifest_result) => update_manifest_result,
                Err(err) => {
                    if err == AppsMgmtError::DownloadNotModified {
                        if app.get_update_state() == AppsUpdateState::Available {
                            let mut app_obj = AppsObject::from(&app);
                            app_obj.allowed_auto_download = is_auto_update;

                            return Ok(Some(app_obj));
                        } else {
                            return Ok(None);
                        }
                    }
                    return Err(AppsServiceError::DownloadManifestFailed);
                }
            };

        if !compare_version_hash(&app, &update_manifest_result.update_manifest) {
            info!("No update available.");
            return Ok(None);
        }

        // Save the downloaded update manifest file in cached dir.
        let cached_dir = webapp_path.join("cached").join(&app.get_name());
        let _ = AppsStorage::ensure_dir(&cached_dir);
        let cached_manifest = cached_dir.join("update.webmanifest");
        if let Err(err) = fs::rename(&update_manifest_result.update_manifest, &cached_manifest) {
            error!(
                "Rename update manifest failed: {} -> {} : {:?}",
                update_manifest_result.update_manifest.display(),
                cached_manifest.display(),
                err
            );
            return Err(AppsServiceError::FilesystemFailure);
        }

        app.set_update_state(AppsUpdateState::Available);
        app.set_manifest_etag(update_manifest_result.manifest_etag);
        // Lock to save app
        {
            let registry = &mut self.shared_data.lock().registry;
            if app.get_update_manifest_url().is_none() {
                app.set_update_manifest_url(AppsItem::new_update_manifest_url(
                    &app.get_name(),
                    registry.get_vhost_port(),
                ));
            }
            let _ = registry.save_app(true, &app)?;
        }

        let mut app_obj = AppsObject::from(&app);
        app_obj.allowed_auto_download = is_auto_update;

        Ok(Some(app_obj))
    }

    pub fn download_icon(
        &self,
        icon: &mut Icon,
        update_url_base: &Url,
        manifest_url_base: &Url,
        download_dir: &Path,
    ) -> Result<(), AppsServiceError> {
        let mut icon_src = icon.get_src();
        // If the icon src is a complete url remove the leading protocol for the download path.
        if let Ok(url) = Url::parse(&icon_src) {
            icon_src = format!("{}{}", url.host().unwrap_or(Domain("")), url.path());
        }
        // If the icon src is an absolute path remove the leading / for the download path.
        // Then it won't end up trying to use a /some/invalid/path/icon.png.
        if icon_src.starts_with('/') {
            let _ = icon_src.remove(0);
        }
        let icon_path = download_dir.join(&icon_src);
        let icon_dir = icon_path.parent().unwrap();
        let icon_url = update_url_base
            .join(&icon.get_src())
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        let _ = AppsStorage::ensure_dir(icon_dir);
        if let Err(err) = self.download(&icon_url, &icon_path) {
            error!(
                "Failed to download icon {} -> {:?} : {:?}",
                icon_url, icon_path, err
            );
            return Err(AppsServiceError::InvalidIcon);
        }
        let icon_cached_url = manifest_url_base
            .join(&icon_src)
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        icon.set_src(icon_cached_url.as_str());

        Ok(())
    }

    pub fn process_deeplinks(
        &mut self,
        apps_item: &mut AppsItem,
        available_dir: &Path,
        manifest: &mut Manifest,
        update_url: &Url,
    ) -> Result<(), AppsServiceError> {
        let b2g_features = match manifest.get_b2g_features() {
            Some(value) => value,
            None => return Ok(()),
        };

        if let Some(deeplinks) = b2g_features.get_deeplinks() {
            let config_url = update_url
                .join(&deeplinks.config())
                .map_err(|_| AppsServiceError::InvalidDeeplinks)?;
            let config_path = available_dir.join("deeplinks_config");

            self.download(&config_url, &config_path)
                .map_err(|_| AppsServiceError::InvalidDeeplinks)?;

            match deeplinks.process(&config_url, &config_path, Some(update_url)) {
                Ok(paths) => {
                    apps_item.set_deeplink_paths(Some(paths));
                    manifest.update_deeplinks(apps_item);
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    pub fn download_and_apply(
        &mut self,
        webapp_path: &Path,
        update_url: &Url,
        is_update: bool,
    ) -> Result<AppsObject, AppsServiceError> {
        let path = Path::new(&webapp_path);
        let update_manifest_result = match self.get_update_manifest(update_url, path, None, true) {
            Ok(update_manifest_result) => update_manifest_result,
            Err(err) => {
                debug!("get_update_manifest err {:?}", err);
                if err == AppsMgmtError::ManifestDownloadFailed(DownloadError::Canceled) {
                    return Err(AppsServiceError::Canceled);
                }
                return Err(AppsServiceError::DownloadManifestFailed);
            }
        };
        let update_manifest =
            UpdateManifest::read_from(update_manifest_result.update_manifest.clone())
                .map_err(|_| AppsServiceError::InvalidManifest)?;

        // Lock registry to do application registration, emit installing event
        // Release lock before downloading zip package. Let downloading happen
        // asyc among other threads
        let mut apps_item: AppsItem;
        let apps_item_restore: AppsItem;
        let mut restorer: AppsStatusRestorer;
        {
            let registry = &mut self.shared_data.lock().registry;
            // Need create appsItem object and add to db to reflect status
            apps_item = if is_update {
                let mut app = registry
                    .get_by_update_url(update_url)
                    .ok_or(AppsServiceError::AppNotFound)?;
                apps_item_restore = app.clone();
                app.set_update_state(AppsUpdateState::Updating);

                app
            } else {
                let app_name = registry.get_unique_name(
                    &update_manifest.get_name(),
                    update_manifest.get_origin(),
                    Some(update_url.clone()),
                )?;
                let mut app = AppsItem::default(&app_name, registry.get_vhost_port());
                app.set_update_url(Some(update_url.clone()));

                apps_item_restore = app.clone();
                app.set_install_state(AppsInstallState::Installing);

                app
            };
            let version = update_manifest.get_version(apps_item.is_pwa());
            if !version.is_empty() {
                debug!("update_manifest version is {}", &version);
                apps_item.set_version(&version);
            }

            apps_item.set_manifest_etag(update_manifest_result.manifest_etag);

            restorer =
                AppsStatusRestorer::new(self.shared_data.clone(), is_update, apps_item_restore);

            let _ = registry.save_app(is_update, &apps_item)?;

            registry.broadcast_installing(is_update, AppsObject::from(&apps_item));
            self.installing_apps_item = Some(apps_item.clone());
        }

        let mut available_dir_remover = DirRemover::new(&update_manifest_result.available_dir);
        let available_dir = update_manifest_result.available_dir;

        let package_url = Url::parse(&update_manifest.get_package_path())
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        if AppsStorage::available_disk_space(webapp_path) < update_manifest.get_packaged_size() * 2
        {
            error!("Do not have enough disk space.");
            return Err(AppsServiceError::DiskSpaceNotEnough);
        }

        let available_zip =
            match self.get_available_zip(update_url, &package_url, is_update, &available_dir) {
                Ok(package) => package,
                Err(err) => {
                    let mut error = Err(AppsServiceError::DownloadPackageFailed);
                    debug!("get_available_zip error {}", err);
                    if err == AppsMgmtError::PackageDownloadFailed(DownloadError::Canceled) {
                        error = Err(AppsServiceError::Canceled);
                    }

                    return error;
                }
            };

        info!("available_zip: {}", available_zip.display());

        let mut manifest =
            validate_package(&available_zip).map_err(|_| AppsServiceError::InvalidPackage)?;

        manifest
            .check_validity()
            .map_err(|_| AppsServiceError::InvalidManifest)?;

        apps_utils::compare_manifests(&update_manifest, &manifest)?;

        if let Err(err) =
            self.process_deeplinks(&mut apps_item, &available_dir, &mut manifest, update_url)
        {
            AppsStorage::log_warn(&format!(
                "Process deeplink for {}, error: {:?}",
                apps_item.get_name(),
                err
            ));
        }

        // We can lock registry now, since no waiting job.
        let shared = &mut self.shared_data.lock();
        let config = shared.config.clone();
        let registry = &mut shared.registry;
        if let Err(err) = verify_zip(&available_zip, &config.cert_type, "inf") {
            error!("Verify zip error: {:?}", err);
            return Err(AppsServiceError::InvalidSignature);
        }

        // Here we can emit ready to apply download if we have sparate steps
        // asking user to apply download
        let _ = registry.apply_download(&mut apps_item, &available_dir, &manifest, is_update)?;

        // Everything went fine, don't remove the available_dir directory.
        available_dir_remover.keep();
        // Everything went fine, keep current app status
        restorer.dont_restore();

        Ok(AppsObject::from(&apps_item))
    }

    pub fn download_and_apply_pwa(
        &mut self,
        webapp_path: &Path,
        update_url: &Url,
        is_update: bool,
    ) -> Result<AppsObject, AppsServiceError> {
        let download_dir = AppsStorage::get_app_dir(
            &webapp_path.join("downloading"),
            &hash(update_url.as_str()).to_string(),
        )
        .map_err(|_| AppsServiceError::DownloadManifestFailed)?;
        let download_manifest = download_dir.join("update.webmanifest");

        // 1. download manfiest to cache dir.
        debug!(
            "download {} to {}",
            update_url.as_str(),
            download_manifest.display()
        );
        if let Err(err) = self.download(update_url, &download_manifest) {
            error!(
                "Downloading {} to {} failed: {:?}",
                update_url,
                download_manifest.display(),
                err
            );
            return Err(AppsServiceError::DownloadManifestFailed);
        }

        let mut manifest = Manifest::read_from(&download_manifest)
            .map_err(|_| AppsServiceError::InvalidManifest)?;
        let update_url_base = update_url.clone();
        let _ = is_same_origin_with(&update_url_base, &manifest.get_start_url())?;
        manifest.process_scope(&update_url_base)?;

        let mut apps_item: AppsItem;
        // Lock registry to do application registration, emit installing event
        {
            let registry = &mut self.shared_data.lock().registry;
            apps_item = if is_update {
                let mut app = registry
                    .get_by_update_url(update_url)
                    .ok_or(AppsServiceError::AppNotFound)?;
                app.set_update_state(AppsUpdateState::Updating);

                app
            } else {
                let app_name = registry.get_unique_name(
                    &manifest.get_name(),
                    manifest.get_origin(),
                    Some(update_url.clone()),
                )?;
                let mut app = AppsItem::default_pwa(&app_name, registry.get_vhost_port());
                app.set_update_url(Some(update_url.clone()));
                app.set_install_state(AppsInstallState::Installing);

                app
            };
            let version = manifest.get_version();
            if !version.is_empty() {
                apps_item.set_version(&version);
            }

            if !is_update {
                registry.validate_pwa_scope(&manifest)?;
            }

            // We make no difference for update or new install at the moment.
            let _ = registry.save_app(false, &apps_item);
            registry.broadcast_installing(is_update, AppsObject::from(&apps_item));
        }
        manifest
            .check_validity()
            .map_err(|_| AppsServiceError::InvalidManifest)?;

        if let Err(err) =
            self.process_deeplinks(&mut apps_item, &download_dir, &mut manifest, update_url)
        {
            AppsStorage::log_warn(&format!(
                "Process deeplink for {}, error: {:?}",
                apps_item.get_name(),
                err
            ));
        }

        // 2-1. download icons to cached dir.
        let manifest_url_base = apps_item.get_manifest_url();
        if let Some(icons_value) = manifest.get_icons() {
            let mut icons: Vec<Icon> =
                serde_json::from_value(icons_value).unwrap_or_else(|_| Vec::new());
            for icon in &mut icons {
                if let Err(err) =
                    icon.process(self, &update_url_base, &manifest_url_base, &download_dir)
                {
                    AppsStorage::log_warn(&format!(
                        "Failed to process icon: {}, err: {:?}",
                        icon.get_src(),
                        err
                    ));
                }
            }
            manifest.set_icons(serde_json::to_value(icons).unwrap());
        }

        // 2-2. update start url in cached manifest to absolute url
        let start_url = update_url_base
            .join(&manifest.get_start_url())
            .map_err(|_| AppsServiceError::InvalidStartUrl)?;
        manifest.set_start_url(start_url.as_str());
        let cached_pwa_manifest = download_dir.join("manifest.webmanifest");
        Manifest::write_to(&cached_pwa_manifest, &manifest)
            .map_err(|_| AppsServiceError::InvalidManifest)?;

        // 2-3: Process if shortcuts exist
        // https://www.w3.org/TR/appmanifest/#dfn-process-the-shortcuts-member
        if let Some(mut shortcuts) = manifest.get_shortcuts() {
            for shortcut in &mut shortcuts {
                if !shortcut.process(&update_url_base) {
                    continue;
                }
                if let Some(mut icons) = shortcut.get_icons() {
                    for icon in &mut icons {
                        if let Err(err) =
                            icon.process(self, &update_url_base, &manifest_url_base, &download_dir)
                        {
                            AppsStorage::log_warn(&format!(
                                "Failed to process icon: {}, err: {:?}",
                                icon.get_src(),
                                err
                            ));
                        }
                    }
                    shortcut.set_icons(Some(icons));
                }
            }
            manifest.set_shortcuts(Some(shortcuts));
        }

        // 3. finish installation and register the pwa app.
        self.shared_data.lock().registry.apply_pwa(
            &mut apps_item,
            &download_dir,
            &manifest,
            is_update,
        )?;

        Ok(AppsObject::from(&apps_item))
    }

    pub fn broadcast_download_failed(&mut self, update_url: &Url, reason: AppsServiceError) {
        let apps_object = if let Some(apps_item) = &self.installing_apps_item {
            AppsObject::from(apps_item)
        } else {
            let mut apps_item = AppsItem::default("unknown", 80);
            apps_item.set_update_url(Some(update_url.clone()));
            AppsObject::from(&apps_item)
        };

        error!("broadcast event: app download failed {:?}", reason);
        self.shared_data
            .lock()
            .registry
            .event_broadcaster
            .broadcast_app_download_failed(DownloadFailedReason {
                apps_object,
                reason,
            });
    }

    fn download<P: AsRef<Path>>(&self, url: &Url, path: P) -> Result<(), DownloadError> {
        let (result_recv, _cancel_sender) = self.downloader.clone().download(url, path, None);
        loop {
            // Wait for 10 mins
            if let Ok(rec) = result_recv.recv_timeout(Duration::from_secs(DOWNLOAD_TIMEOUT)) {
                match rec {
                    Ok(info) => {
                        if let DownloaderInfo::Done = info {
                            break;
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else {
                return Err(DownloadError::Other("Timed Out".into()));
            }
        }

        Ok(())
    }

    fn set_or_update_canceller(&mut self, url: &Url, cancel_sender: Sender<()>) {
        let downloadings = &mut self.shared_data.lock().downloadings;
        downloadings
            .entry(url.to_owned())
            .and_modify(|canceller| canceller.cancel_sender = Some(cancel_sender.clone()))
            .or_insert(DownloadingCanceller {
                cancel_sender: Some(cancel_sender),
                cancelled: false,
            });
    }

    fn check_cancelled(&self, url: &Url) -> bool {
        let downloadings = &self.shared_data.lock().downloadings;
        downloadings
            .get(url)
            .map(|canceller| canceller.cancelled)
            .unwrap_or(false)
    }
}

// Returns success if both urls are absolute and same origin, or if the `other`
// url is a relative one.
fn is_same_origin_with(base_url: &Url, other: &str) -> Result<(), AppsServiceError> {
    // We always return InvalidManifest for convenience instead of dedicated error variants.
    if let Ok(other_url) = Url::parse(other) {
        if other_url.origin() != base_url.origin() {
            Err(AppsServiceError::InvalidManifest)
        } else {
            Ok(())
        }
    } else {
        // This is a relative url, which is always same origin with the base.
        Ok(())
    }
}

fn get_etag_header(app: &AppsItem, headers: &mut HeaderMap) {
    if let Some(etag) = app.get_manifest_etag() {
        let etag_header_value = match HeaderValue::from_str(&etag) {
            Ok(val) => val,
            Err(_) => {
                return;
            }
        };
        headers.insert(
            HeaderName::from_lowercase(b"if-none-match").unwrap(),
            etag_header_value,
        );
    }
}

fn compute_manifest_hash<P: AsRef<Path>>(p: P) -> Result<String, AppsError> {
    let mut buffer = Vec::new();
    let mut file = File::open(p)?;
    if file.read_to_end(&mut buffer).is_ok() {
        let mut hasher = Blake2s256::new();
        hasher.update(&buffer);
        let res = hasher.finalize();
        Ok(format!("{:x}", res))
    } else {
        Ok(String::new())
    }
}

pub fn is_new_version(old_version: &str, new_version: &str) -> bool {
    if old_version.is_empty() || new_version.is_empty() {
        return false;
    }
    if let (Some(new_version), Some(old_version)) =
        (Version::from(new_version), Version::from(old_version))
    {
        return new_version.compare(&old_version) == CompOp::Gt;
    }
    false
}

fn compare_version_hash<P: AsRef<Path>>(app: &AppsItem, update_manifest: P) -> bool {
    let manifest = match UpdateManifest::read_from(&update_manifest) {
        Ok(manifest) => manifest,
        Err(_) => return false,
    };

    debug!("from update manifest{:#?}", manifest);
    debug!("hash from registry {}", app.get_manifest_hash());

    let mut is_update_available = false;
    let app_version = app.get_version();
    let manifest_version = manifest.get_version(app.is_pwa());
    if app_version.is_empty() || manifest_version.is_empty() {
        let hash_str = match compute_manifest_hash(&update_manifest) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        debug!("hash from update manifest {}", hash_str);
        if hash_str != app.get_manifest_hash() {
            is_update_available = true;
        }
    } else {
        is_update_available = is_new_version(&app_version, &manifest_version);
    }

    debug!("compare_version_hash update {}", is_update_available);
    is_update_available
}

fn remove_canceller(shared_data: Shared<AppsSharedData>, url: &Url) {
    let _ = shared_data.lock().downloadings.remove(url);
}

#[cfg(test)]
fn test_apply_pwa(app_url_str: &str, expected_err: Option<AppsServiceError>) {
    use crate::apps_registry::AppsRegistry;
    use crate::config;
    use config::Config;
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir-pwa.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir-pwa", current.display());
    let test_path = Path::new(&test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    let _ = remove_dir_all(&test_path);

    println!("Register from: {}", &root_path);
    let config = Config::new(
        root_path,
        test_dir.clone(),
        String::from("uds_path"),
        String::from("test"),
        String::from("updater_socket"),
        true,
    );

    // Create a locale instance of shared data because it's using a different
    // configuration path than other tests.
    let shared_data: Shared<AppsSharedData> = Shared::default();
    let vhost_port = 80;
    match AppsRegistry::initialize(&config, vhost_port) {
        Ok(registry) => {
            shared_data.lock().registry = registry;
        }
        Err(err) => {
            panic!("Failed to initialize registry: {:?}", err);
        }
    };

    println!("registry.count(): {}", shared_data.lock().registry.count());
    assert_eq!(6, shared_data.lock().registry.count());

    // Test 1: apply from a local dir
    let src_manifest = current.join("test-fixtures/apps-from/pwa/manifest.webmanifest");
    let update_url = Url::parse("https://pwa1.test/manifest.webmanifest").unwrap();
    let download_dir = AppsStorage::get_app_dir(
        &test_path.join("downloading"),
        &format!("{}", hash(update_url.as_str())),
    )
    .unwrap();
    let download_manifest = download_dir.join("manifest.webmanifest");

    if let Err(err) = std::fs::create_dir_all(&download_dir) {
        println!("{:?}", err);
    }
    let _ = std::fs::copy(&src_manifest, &download_manifest).unwrap();
    let manifest = Manifest::read_from(&download_manifest).unwrap();
    let app_name = shared_data
        .lock()
        .registry
        .get_unique_name(
            &manifest.get_name(),
            manifest.get_origin(),
            Some(update_url.clone()),
        )
        .unwrap();
    if let Some(icons_value) = manifest.get_icons() {
        let icons: Vec<Icon> = serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
        assert_eq!(4, icons.len());
    } else {
        panic!();
    }
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    apps_item.set_install_state(AppsInstallState::Installing);
    apps_item.set_update_url(Some(update_url.clone()));

    match shared_data
        .lock()
        .registry
        .apply_pwa(&mut apps_item, &download_dir, &manifest, false)
    {
        Ok(_) => {
            assert_eq!(apps_item.get_name(), "helloworld");
            assert_eq!(apps_item.get_install_state(), AppsInstallState::Installed);
        }
        Err(err) => {
            println!("err: {:?}", err);
            panic!();
        }
    }

    // Test 2: download and apply from a remote url
    let mut request = AppsRequest::new(shared_data.clone()).unwrap();
    let app_url = Url::parse(app_url_str).unwrap();
    match request.download_and_apply_pwa(test_path, &app_url, false) {
        Ok(app) => {
            if expected_err.is_some() {
                panic!();
            }
            assert_eq!(app.name, "hellopwa");
            assert_eq!(app.removable, true);
        }
        Err(err) => {
            if let Some(expected) = expected_err {
                if err == expected {
                    return;
                }
            }
            println!("err: {:?}", err);
            panic!();
        }
    }
    let lock = shared_data.lock();
    if let Some(app) = lock.registry.get_by_update_url(&app_url) {
        assert_eq!(app.get_name(), "hellopwa");

        let cached_dir = test_path.join("cached");
        let update_manifest = cached_dir.join(app.get_name()).join("manifest.webmanifest");
        let manifest = Manifest::read_from(&update_manifest).unwrap();

        // The start url in cached manifest is an absolute url of remote address.
        let expected_start_url = app_url.join("index.html").unwrap().as_str().to_string();
        assert_eq!(manifest.get_start_url(), expected_start_url);

        // icon url should be relative path of local cached address
        if let Some(icons_value) = manifest.get_icons() {
            let icons: Vec<Icon> = serde_json::from_value(icons_value).unwrap_or_else(|_| vec![]);
            assert_eq!(4, icons.len());
            let manifest_url_base = app.get_manifest_url().join("/");
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
}

#[test]
fn test_compare_version_hash() {
    use std::env;

    let _ = env_logger::try_init();

    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/compare-version-hash/sample_update_manifest.webmanifest",
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
        apps_item
            .set_manifest_hash("9e61057bbf5fxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxe7ff60c78804727917009");

        assert!(compare_version_hash(&apps_item, &manifest_path));
    }

    // hash
    {
        let app_name = "test";
        let mut apps_item = AppsItem::default(&app_name, 443);
        apps_item
            .set_manifest_hash("9e61057bbf5f11073bb19fc30db81076811c27bb9cbe7ff60c78804727917009");

        assert!(!compare_version_hash(&apps_item, &manifest_path));
    }
}

#[test]
fn test_is_same_origin_with() {
    let base_url = Url::parse("https://domain.url/path/file").unwrap();

    assert_eq!(is_same_origin_with(&base_url, "index.html"), Ok(()));
    assert_eq!(is_same_origin_with(&base_url, "/app/index.html"), Ok(()));
    assert_eq!(
        is_same_origin_with(&base_url, "https://domain.url/index.html"),
        Ok(())
    );
    assert_eq!(
        is_same_origin_with(&base_url, "https://domain.url/app/index.html"),
        Ok(())
    );
    assert_eq!(
        is_same_origin_with(&base_url, "https://other.url/index.html"),
        Err(AppsServiceError::InvalidManifest)
    );
    assert_eq!(
        is_same_origin_with(&base_url, "https://other.url/app/index.html"),
        Err(AppsServiceError::InvalidManifest)
    );
}
