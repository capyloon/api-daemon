// Implementation of the tasks matching api calls.

use crate::apps_registry::AppMgmtTask;
use crate::generated::common::*;
use crate::shared_state::AppsSharedData;
use common::traits::Shared;
use log::{error, info};

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
                responder.resolve(app.clone());
                shared.registry.event_broadcaster.broadcast_app_updated(app);
            }
            Err(err) => {
                info!("broadcast event: app_updated failed. Reason: {:?}", err);
                shared
                    .registry
                    .event_broadcaster
                    .broadcast_app_download_failed(AppsObject::from(old_app));
                responder.reject(err);
            }
        }
    }
}

pub struct CheckForUpdateTask(
    pub Shared<AppsSharedData>,
    pub String,                            // Update url
    pub AppsOptions,                       // For auto update option
    pub AppsEngineCheckForUpdateResponder, // responder
);

impl AppMgmtTask for CheckForUpdateTask {
    fn run(&self) {
        let mut shared = self.0.lock();
        let url = &self.1;
        let apps_option = &self.2;
        let responder = &self.3;

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
                if let Some(app) = ret {
                    shared
                        .registry
                        .event_broadcaster
                        .broadcast_app_update_available(app);
                    responder.resolve(true);
                } else {
                    responder.resolve(false);
                }
            }
            Err(err) => {
                info!("CheckForUpdateTask error {:?}", err);
                responder.reject(err);
            }
        }
    }
}
