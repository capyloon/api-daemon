// Implementation of the tasks matching api calls.

use crate::apps_registry::AppMgmtTask;
use crate::apps_request::AppsRequest;
use crate::generated::common::*;
use crate::shared_state::AppsSharedData;
use common::traits::Shared;
use log::{error, info};
use std::env;

pub struct InstallPackageTask(
    pub Shared<AppsSharedData>,
    pub String,                            // Update url
    pub AppsEngineInstallPackageResponder, // responder
);

impl AppMgmtTask for InstallPackageTask {
    fn run(&self) {
        // Actually run the installation.
        let shared_data = self.0.clone();
        let request = AppsRequest::new(shared_data);
        let url = &self.1;
        let responder = &self.2;

        if request.is_err() {
            return responder.reject(AppsServiceError::UnknownError);
        }
        let mut request = request.unwrap();
        if let Some(app) = request.shared_data.lock().registry.get_by_update_url(&url) {
            if app.get_install_state() == AppsInstallState::Installed
                || app.get_install_state() == AppsInstallState::Installing
            {
                return responder.reject(AppsServiceError::ReinstallForbidden);
            }
        }

        let _ = ensure_token_deviceinfo(&mut request);

        let data_path = request.shared_data.lock().config.data_path.clone();
        let current = env::current_dir().unwrap();
        let data_dir = current.join(data_path);
        match request.download_and_apply(data_dir.to_str().unwrap(), &url, false) {
            Ok(app) => {
                info!("broadcast event: app_installed");
                let mut shared = request.shared_data.lock();
                shared.vhost_api.app_installed(&app.name);
                responder.resolve(app.clone());
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_installed(app);
            }
            Err(err) => {
                request.broadcast_download_failed(&url, err);
                responder.reject(err);
            }
        }
    }
}

pub struct InstallPwaTask(
    pub Shared<AppsSharedData>,
    pub String,                        // Update url
    pub AppsEngineInstallPwaResponder, // responder
);

impl AppMgmtTask for InstallPwaTask {
    fn run(&self) {
        // Actually run the installation.
        let shared_data = self.0.clone();
        let request = AppsRequest::new(shared_data);
        let url = &self.1;
        let responder = &self.2;

        if request.is_err() {
            return responder.reject(AppsServiceError::UnknownError);
        }
        let mut request = request.unwrap();
        if request
            .shared_data
            .lock()
            .registry
            .get_by_update_url(&url)
            .is_some()
        {
            return responder.reject(AppsServiceError::ReinstallForbidden);
        }

        let data_path = request.shared_data.lock().config.data_path.clone();
        match request.download_and_apply_pwa(&data_path, &url, false) {
            Ok(app) => {
                info!("broadcast event: app_installed");
                responder.resolve(app.clone());
                request
                    .shared_data
                    .lock()
                    .registry
                    .event_broadcaster
                    .broadcast_app_installed(app);
            }
            Err(err) => {
                request.broadcast_download_failed(url, err);
                responder.reject(err);
            }
        }
    }
}

pub struct UninstallTask(
    pub Shared<AppsSharedData>,
    pub String,                       // Manifest url
    pub AppsEngineUninstallResponder, // Responder.
);

impl AppMgmtTask for UninstallTask {
    fn run(&self) {
        let shared_data = self.0.clone();
        let request = AppsRequest::new(shared_data);
        let url = &self.1;
        let responder = &self.2;

        if request.is_err() {
            return responder.reject(AppsServiceError::UnknownError);
        }
        let request = request.unwrap();
        let app = match request.shared_data.lock().get_by_manifest_url(&url) {
            Ok(app) => app,
            Err(err) => {
                error!("Do not find uninstall app: {:?}", err);
                return responder.reject(AppsServiceError::AppNotFound);
            }
        };

        let mut shared = request.shared_data.lock();
        let data_path = shared.config.data_path.clone();
        let current = env::current_dir().unwrap();
        let data_dir = current.join(data_path);
        if let Err(err) = shared
            .registry
            .uninstall_app(&app.manifest_url, data_dir.to_str().unwrap())
        {
            error!("Unregister app failed: {:?}", err);
            return responder.reject(err);
        }

        shared.vhost_api.app_uninstalled(&app.name);
        responder.resolve(app.manifest_url.clone());
        shared
            .registry
            .event_broadcaster
            .broadcast_app_uninstalled(app.manifest_url);
    }
}

pub struct UpdateTask(
    pub Shared<AppsSharedData>,
    pub String,                    // Manifest url
    pub AppsEngineUpdateResponder, // Responder.
);

impl AppMgmtTask for UpdateTask {
    fn run(&self) {
        let shared_data = self.0.clone();
        let request = AppsRequest::new(shared_data);
        let url = &self.1;
        let responder = &self.2;

        if request.is_err() {
            return responder.reject(AppsServiceError::UnknownError);
        }
        let mut request = request.unwrap();
        let old_app = match request
            .shared_data
            .lock()
            .registry
            .get_by_manifest_url(&url)
        {
            Some(app) => app,
            None => {
                error!("Update app not found: {}", url);
                return responder.reject(AppsServiceError::AppNotFound);
            }
        };

        let _ = ensure_token_deviceinfo(&mut request);

        let update_url = old_app.get_update_url();
        let data_path = request.shared_data.lock().config.data_path.clone();
        let current = env::current_dir().unwrap();
        let data_dir = current.join(&data_path);

        let update_result = if old_app.is_pwa() {
            request.download_and_apply_pwa(data_dir.to_str().unwrap(), &update_url, true)
        } else {
            request.download_and_apply(data_dir.to_str().unwrap(), &update_url, true)
        };

        match update_result {
            Ok(app) => {
                info!("broadcast event: app_updated");
                let mut shared = request.shared_data.lock();
                shared.vhost_api.app_updated(&app.name);
                responder.resolve(app.clone());
                shared.registry.event_broadcaster.broadcast_app_updated(app);
            }
            Err(err) => {
                request.broadcast_download_failed(&update_url, err);
                responder.reject(err);
            }
        }
    }
}

pub struct CheckForUpdateTask(
    pub Shared<AppsSharedData>,
    pub String,                                    // Update url
    pub AppsOptions,                               // For auto update option
    pub Option<AppsEngineCheckForUpdateResponder>, // some responder
);

impl AppMgmtTask for CheckForUpdateTask {
    fn run(&self) {
        let shared_data = self.0.clone();
        let request = AppsRequest::new(shared_data);
        let url = &self.1;
        let apps_option = &self.2;
        let some_responder = &self.3;

        if request.is_err() {
            if let Some(responder) = some_responder {
                return responder.reject(AppsServiceError::UnknownError);
            }
        }
        let mut request = request.unwrap();

        let _ = ensure_token_deviceinfo(&mut request);

        let data_path = request.shared_data.lock().config.data_path.clone();
        let is_auto_update = apps_option.auto_install.unwrap_or(false);
        match request.check_for_update(&data_path, &url, is_auto_update) {
            Ok(ret) => {
                info!("broadcast event: check_for_update");
                let mut updated = false;
                if let Some(app) = ret {
                    request
                        .shared_data
                        .lock()
                        .registry
                        .event_broadcaster
                        .broadcast_app_update_available(app);
                    updated = true;
                }

                if let Some(responder) = some_responder {
                    responder.resolve(updated);
                }
            }
            Err(err) => {
                error!("CheckForUpdateTask error {:?}", err);
                if let Some(responder) = some_responder {
                    responder.reject(err);
                }
            }
        }
    }
}

pub struct SetEnabledTask(
    pub Shared<AppsSharedData>,
    pub String,                        // manifest url
    pub AppsStatus,                    // App status
    pub AppsEngineSetEnabledResponder, // responder
);

impl AppMgmtTask for SetEnabledTask {
    fn run(&self) {
        let mut shared = self.0.lock();
        let manifest_url = &self.1;
        let status = self.2;
        let responder = &self.3;
        let current = env::current_dir().unwrap();
        let config = shared.config.clone();
        let data_path = current.join(config.data_path);
        match shared
            .registry
            .set_enabled(&manifest_url, status, &data_path)
        {
            Ok((app, changed)) => {
                if changed {
                    if status == AppsStatus::Disabled {
                        shared.vhost_api.app_disabled(&app.name);
                    }
                    shared
                        .registry
                        .event_broadcaster
                        .broadcast_appstatus_changed(app.clone());
                }
                responder.resolve(app);
            }
            Err(err) => responder.reject(err),
        };
    }
}

pub struct ClearTask(
    pub Shared<AppsSharedData>,
    pub String,                   // manifest url
    pub ClearType,                // App status
    pub AppsEngineClearResponder, // responder
);

impl AppMgmtTask for ClearTask {
    fn run(&self) {
        let mut shared = self.0.lock();
        let manifest_url = &self.1;
        let datatype = self.2;
        let responder = &self.3;

        if shared.registry.get_by_manifest_url(&manifest_url).is_none() {
            return responder.reject(AppsServiceError::AppNotFound);
        }

        let data_path = shared.config.data_path.clone();
        match shared.registry.clear(manifest_url, datatype, &data_path) {
            Ok(_) => responder.resolve(true),
            Err(err) => responder.reject(err),
        };
    }
}

pub struct CancelDownloadTask(
    pub Shared<AppsSharedData>,
    pub String,                            // update url
    pub AppsEngineCancelDownloadResponder, // responder
);

impl AppMgmtTask for CancelDownloadTask {
    fn run(&self) {
        let mut shared = self.0.lock();
        let update_url = &self.1;
        let responder = &self.2;

        info!("cancel dwonload {}", &update_url);
        let app = match shared.registry.get_by_update_url(&update_url) {
            Some(app) => app,
            None => return responder.reject(AppsServiceError::AppNotFound),
        };

        match shared.downloadings.get(update_url) {
            Some(cancel_sender) => {
                let _ = cancel_sender.send(());
                shared.downloadings.remove(update_url);
                responder.resolve(AppsObject::from(&app));
            }
            None => responder.reject(AppsServiceError::AppNotFound),
        };
    }
}

// This function blocks current thread to receive token
// Used only in threadpool
fn ensure_token_deviceinfo(_request: &mut AppsRequest) -> bool {
    // Make if believe it's always true.
    true
}
