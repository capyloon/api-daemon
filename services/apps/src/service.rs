// Implementation of the apps service
use super::shared_state::*;
use crate::generated::common::*;
use crate::generated::service::*;
use crate::tasks::{
    CancelDownloadTask, CheckForUpdateTask, ClearTask, InstallPackageTask, InstallPwaTask,
    SetEnabledTask, UninstallTask, UpdateTask,
};
use crate::update_scheduler::{SchedulerMessage, UpdateScheduler};
use common::core::BaseMessage;
use common::traits::{
    DispatcherId, OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext,
    TrackerId,
};
use log::{debug, error, info};
use std::collections::HashMap;
use zip_utils::verify_zip;

pub struct AppsService {
    id: TrackerId,
    proxy_tracker: AppsManagerProxyTracker,
    pub shared_data: Shared<AppsSharedData>,
    dispatcher_id: DispatcherId,
}

impl AppsManager for AppsService {
    fn get_proxy_tracker(&mut self) -> &mut AppsManagerProxyTracker {
        &mut self.proxy_tracker
    }
}

impl AppsEngineMethods for AppsService {
    fn get_all(&mut self, responder: &AppsEngineGetAllResponder) {
        info!("get_all");
        let shared = self.shared_data.lock();
        match shared.get_all_apps() {
            Ok(apps) => {
                if apps.is_empty() {
                    responder.resolve(None);
                } else {
                    responder.resolve(Some(apps));
                }
            }
            Err(err) => responder.reject(err),
        }
    }

    fn get_app(&mut self, responder: &AppsEngineGetAppResponder, manifest_url: String) {
        info!("get_app: {}", manifest_url);
        let shared = self.shared_data.lock();
        match shared.get_by_manifest_url(&manifest_url) {
            Ok(app) => responder.resolve(app),
            Err(err) => responder.reject(err),
        }
    }

    fn get_state(&mut self, responder: &AppsEngineGetStateResponder) {
        info!("get_state");
        let shared = self.shared_data.lock();
        responder.resolve(shared.state);
    }

    fn install_package(
        &mut self,
        responder: &AppsEngineInstallPackageResponder,
        update_url: String,
    ) {
        info!("install_package: {}", &update_url);
        let task = InstallPackageTask(self.shared_data.clone(), update_url, responder.clone());
        self.shared_data.lock().registry.queue_task(task);
    }

    fn check_for_update(
        &mut self,
        responder: &AppsEngineCheckForUpdateResponder,
        update_url: String,
        apps_option: AppsOptions,
    ) {
        info!("check_for_update: {}", &update_url);
        let task = CheckForUpdateTask(
            self.shared_data.clone(),
            update_url,
            apps_option,
            Some(responder.clone()),
        );
        self.shared_data.lock().registry.queue_task(task);
    }

    fn cancel_download(
        &mut self,
        responder: &AppsEngineCancelDownloadResponder,
        update_url: String,
    ) {
        info!("cancel_download: {}", &update_url);
        let task = CancelDownloadTask(self.shared_data.clone(), update_url, responder.clone());
        self.shared_data.lock().registry.queue_task(task);
    }

    fn install_pwa(&mut self, responder: &AppsEngineInstallPwaResponder, update_url: String) {
        info!("install_pwa: {}", &update_url);
        let task = InstallPwaTask(self.shared_data.clone(), update_url, responder.clone());
        self.shared_data.lock().registry.queue_task(task);
    }

    fn update(&mut self, responder: &AppsEngineUpdateResponder, manifest_url: String) {
        info!("update: {}", &manifest_url);
        let task = UpdateTask(self.shared_data.clone(), manifest_url, responder.clone());
        self.shared_data.lock().registry.queue_task(task);
    }

    fn uninstall(&mut self, responder: &AppsEngineUninstallResponder, manifest_url: String) {
        info!("uninstall: {}", &manifest_url);
        let task = UninstallTask(self.shared_data.clone(), manifest_url, responder.clone());
        self.shared_data.lock().registry.queue_task(task);
    }

    fn set_enabled(
        &mut self,
        responder: &AppsEngineSetEnabledResponder,
        manifest_url: String,
        status: AppsStatus,
    ) {
        info!("set_enabled: {:?}, for {}", &status, &manifest_url);
        let task = SetEnabledTask(
            self.shared_data.clone(),
            manifest_url,
            status,
            responder.clone(),
        );
        self.shared_data.lock().registry.queue_task(task);
    }

    fn set_update_policy(
        &mut self,
        responder: &AppsEngineSetUpdatePolicyResponder,
        config: UpdatePolicy,
    ) {
        info!(
            "set_update_policy: {}, {:?}, {}",
            config.enabled, &config.conn_type, config.delay
        );
        if let Some(sender) = &self.shared_data.lock().scheduler {
            if let Ok(result) = sender.send(SchedulerMessage::Config(config)) {
                info!("scheduler.send success {:?}", result);
                responder.resolve(true);
            } else {
                responder.resolve(false);
            }
        } else {
            responder.resolve(false);
        }
    }

    fn get_update_policy(&mut self, responder: &AppsEngineGetUpdatePolicyResponder) {
        info!("get_update_policy");
        let scheduler = UpdateScheduler::new();

        responder.resolve(scheduler.get_config());
    }

    fn clear(
        &mut self,
        responder: &AppsEngineClearResponder,
        manifest_url: String,
        data_type: ClearType,
    ) {
        info!("clear: {}", &manifest_url);
        let task = ClearTask(
            self.shared_data.clone(),
            manifest_url,
            data_type,
            responder.clone(),
        );
        self.shared_data.lock().registry.queue_task(task);
    }

    fn verify(
        &mut self,
        responder: &AppsEngineVerifyResponder,
        manifest_url: String,
        cert_type: String,
        folder_name: String,
    ) {
        info!("verify {}, {}, {}", manifest_url, cert_type, folder_name);
        let shared = self.shared_data.lock();
        let data_path = shared.config.data_path.clone();
        let package_path = match shared.registry.get_pacakge_path(&data_path, &manifest_url) {
            Ok(path) => path,
            Err(err) => {
                return responder.reject(err);
            }
        };

        match verify_zip(package_path, &cert_type, &folder_name) {
            Ok(fingerprint) => {
                debug!("verify success rsa cert fingerprint sha1 {}", &fingerprint);
                responder.resolve(fingerprint);
            }
            Err(_) => {
                responder.reject(AppsServiceError::InvalidSignature);
            }
        }
    }

    fn set_token_provider(
        &mut self,
        responder: &AppsEngineSetTokenProviderResponder,
        provider: ObjectRef,
    ) {
        info!("set_token_provider");
        if let Some(AppsManagerProxy::TokenProvider(token_provider)) =
            self.proxy_tracker.get(&provider)
        {
            self.shared_data
                .lock()
                .token_provider
                .set_provider(token_provider.clone());
            responder.resolve();
        } else {
            error!("Failed to get token proxy_tracker");
            responder.reject();
        }
    }
}

impl Service<AppsService> for AppsService {
    // Shared among instances.
    type State = AppsSharedData;

    fn shared_state() -> Shared<Self::State> {
        let shared = &*APPS_SHARED_SHARED_DATA;
        shared.clone()
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        shared_data: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Result<AppsService, String> {
        info!("AppsService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = AppsEngineEventDispatcher::from(helper, 0 /* object id */);
        let dispatcher_id = shared_data
            .lock()
            .registry
            .add_dispatcher(&event_dispatcher);
        Ok(AppsService {
            id: service_id,
            proxy_tracker: HashMap::new(),
            shared_data,
            dispatcher_id,
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<AppsManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("AppsService request: {:?}", req),
            Err(err) => format!("Unable to format AppsService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        true
    }
}

impl Drop for AppsService {
    fn drop(&mut self) {
        info!("Dropping Apps Service #{}", self.id);
        self.shared_data
            .lock()
            .registry
            .remove_dispatcher(self.dispatcher_id);
    }
}
