// Implementation of the tasks matching api calls.

use crate::apps_registry::AppMgmtTask;
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
        let mut shared = self.0.lock();
        let url = &self.1;
        let responder = &self.2;

        if let Some(app) = shared.registry.get_by_update_url(&url) {
            if app.get_install_state() == AppsInstallState::Installed
                || app.get_install_state() == AppsInstallState::Installing
            {
                info!("broadcast event: app_download_failed");
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_download_failed(AppsObject::from(&app));
                return responder.reject(AppsServiceError::ReinstallForbidden);
            }
        }

        let data_path = shared.config.data_path.clone();
        match shared.registry.download_and_apply(&data_path, &url, false) {
            Ok(app) => {
                info!("broadcast event: app_installed");
                shared.vhost_api.app_installed(&app.name);
                responder.resolve(app.clone());
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_installed(app);
            }
            Err(err) => {
                info!("InstallPackageTask error download_and_apply {:?}", err);
                if let Some(app) = shared.registry.get_by_update_url(&url) {
                    if app.get_install_state() == AppsInstallState::Installed
                        || app.get_install_state() == AppsInstallState::Installing
                    {
                        shared
                            .registry
                            .event_broadcaster
                            .broadcast_app_download_failed(AppsObject::from(&app));
                    }
                }
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
        let mut shared = self.0.lock();
        let url = &self.1;
        let responder = &self.2;

        if shared.registry.get_by_update_url(&url).is_some() {
            return responder.reject(AppsServiceError::ReinstallForbidden);
        }
        let data_path = shared.config.data_path.clone();
        match shared.registry.download_and_apply_pwa(&data_path, &url) {
            Ok(app) => {
                info!("broadcast event: app_installed");
                responder.resolve(app.clone());
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_installed(app);
            }
            Err(err) => {
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
        let mut shared = self.0.lock();
        let url = &self.1;
        let responder = &self.2;

        let app = match shared.get_by_manifest_url(&url) {
            Ok(app) => app,
            Err(err) => {
                error!("Do not find uninstall app: {:?}", err);
                return responder.reject(AppsServiceError::AppNotFound);
            }
        };

        let data_path = shared.config.data_path.clone();
        if let Err(err) = shared
            .registry
            .uninstall_app(&app.name, &app.update_url, &data_path)
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
        let mut shared = self.0.lock();
        let url = &self.1;
        let responder = &self.2;

        let old_app = match shared.get_by_manifest_url(&url) {
            Ok(app) => app,
            Err(err) => {
                error!("Update app not found: {:?}", err);
                return responder.reject(AppsServiceError::AppNotFound);
            }
        };

        let update_url = &old_app.update_url;

        let data_path = shared.config.data_path.clone();
        match shared
            .registry
            .download_and_apply(&data_path, &update_url, true)
        {
            Ok(app) => {
                info!("broadcast event: app_updated");
                shared.vhost_api.app_updated(&app.name);
                responder.resolve(app.clone());
                shared.registry.event_broadcaster.broadcast_app_updated(app);
            }
            Err(err) => {
                info!("broadcast event: app_updated failed. Reason: {:?}", err);
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_download_failed(old_app);
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
        let mut shared = self.0.lock();
        let url = &self.1;
        let apps_option = &self.2;
        let some_responder = &self.3;

        let data_path = shared.config.data_path.clone();
        let is_auto_update = match apps_option.auto_install {
            Some(is_auto) => is_auto,
            None => false,
        };
        match shared
            .registry
            .check_for_update(&data_path, &url, is_auto_update)
        {
            Ok(ret) => {
                info!("broadcast event: check_for_update");
                let mut updated = false;
                if let Some(app) = ret {
                    shared
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
                info!("CheckForUpdateTask error {:?}", err);
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
        let root_path = current.join(shared.config.root_path.clone());
        let data_path = current.join(shared.config.data_path.clone());
        match shared
            .registry
            .set_enabled(&manifest_url, status, &data_path, &root_path)
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

        match shared.registry.clear(manifest_url, datatype) {
            Ok(_) => responder.resolve(true),
            Err(err) => responder.reject(err),
        };
    }
}
