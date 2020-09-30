// Implementation of the apps service
use super::shared_state::*;
use crate::generated::common::*;
use crate::generated::service::*;
use crate::tasks::{CheckForUpdateTask, InstallPackageTask, UninstallTask, UpdateTask};
use common::core::BaseMessage;
use common::traits::{
    DispatcherId, OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext,
    TrackerId,
};
use log::info;

#[derive(Clone)]
pub struct AppsService {
    id: TrackerId,
    pub shared_data: Shared<AppsSharedData>,
    event_dispatcher: AppsEngineEventDispatcher,
    dispatcher_id: DispatcherId,
}

impl AppsManager for AppsService {}

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
            responder.clone(),
        );
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
    ) -> Option<AppsService> {
        info!("AppsService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = AppsEngineEventDispatcher::from(helper, 0 /* object id */);
        let dispatcher_id = shared_data
            .lock()
            .registry
            .add_dispatcher(&event_dispatcher);
        Some(AppsService {
            id: service_id,
            shared_data,
            event_dispatcher,
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
