/// Implementation of the test service.
use crate::db::{ObserverType, SettingsDb};
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use common::threadpool_status;
use common::traits::{
    CommonResponder, DispatcherId, EmptyConfig, OriginAttributes, Service, SessionSupport, Shared,
    SharedServiceState, SharedSessionContext, StateLogger, TrackerId,
};
use log::{error, info};
use std::collections::HashMap;
use threadpool::ThreadPool;

fn can_access_setting(setting: &str, attributes: &OriginAttributes) -> bool {
    if attributes.has_permission("settings:read") || attributes.has_permission("settings:write") {
        return true;
    }

    if setting == "nutria.theme" && attributes.has_permission("themeable") {
        return true;
    }

    false
}

pub struct SettingsSharedData {
    pub db: SettingsDb,
    pub pool: ThreadPool,
}

impl StateLogger for SettingsSharedData {
    fn log(&self) {
        self.db.log();
        info!("  Threadpool {}", threadpool_status(&self.pool));
    }
}

impl From<&EmptyConfig> for SettingsSharedData {
    fn from(_config: &EmptyConfig) -> Self {
        SettingsSharedData {
            db: SettingsDb::new(SettingsFactoryEventBroadcaster::default()),
            pool: ThreadPool::with_name("SettingsService".into(), 5),
        }
    }
}

pub struct SettingsService {
    id: TrackerId,
    proxy_tracker: SettingsManagerProxyTracker,
    state: Shared<SettingsSharedData>,
    dispatcher_id: Option<DispatcherId>,
    observers: HashMap<ObjectRef, Vec<(String, DispatcherId)>>,
    origin_attributes: OriginAttributes,
    pool: ThreadPool,
}

impl SettingsManager for SettingsService {
    fn get_proxy_tracker(&mut self) -> &mut SettingsManagerProxyTracker {
        &mut self.proxy_tracker
    }
}

impl SettingsFactoryMethods for SettingsService {
    fn clear(&mut self, responder: SettingsFactoryClearResponder) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "settings:write",
            "clear settings",
        ) {
            return;
        }

        match self.state.lock().db.clear() {
            Ok(()) => responder.resolve(),
            Err(_) => responder.reject(),
        }
    }

    fn get(&mut self, responder: SettingsFactoryGetResponder, name: String) {
        if !can_access_setting(&name, &self.origin_attributes)
            && responder.maybe_send_permission_error(
                &self.origin_attributes,
                "settings:read",
                "get setting",
            )
        {
            return;
        }

        let shared = self.state.clone();
        self.pool.execute(move || {
            let db = &shared.lock().db;
            match db.get(&name) {
                Ok(value) => responder.resolve(SettingInfo { name, value }),
                Err(crate::db::Error::Sqlite(rusqlite::Error::QueryReturnedNoRows)) => responder
                    .reject(GetError {
                        name,
                        reason: GetErrorReason::NonExistingSetting,
                    }),
                Err(err) => {
                    error!("db get error {:?}", err);
                    responder.reject(GetError {
                        name,
                        reason: GetErrorReason::UnknownError,
                    })
                }
            }
        });
    }

    fn set(&mut self, responder: SettingsFactorySetResponder, settings: Vec<SettingInfo>) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "settings:write",
            "set settings",
        ) {
            return;
        }

        let shared = self.state.clone();
        self.pool.execute(move || {
            let db = &mut shared.lock().db;
            match db.set(&settings) {
                Ok(_) => responder.resolve(),
                Err(_) => responder.reject(),
            }
        });
    }

    fn get_batch(&mut self, responder: SettingsFactoryGetBatchResponder, names: Vec<String>) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "settings:read",
            "get a batch of settings",
        ) {
            return;
        }

        let shared = self.state.clone();
        self.pool.execute(move || {
            let db = &shared.lock().db;
            match db.get_batch(&names) {
                Ok(values) => responder.resolve(values),
                Err(_) => responder.reject(),
            }
        });
    }

    fn add_observer(
        &mut self,
        responder: SettingsFactoryAddObserverResponder,
        name: String,
        observer: ObjectRef,
    ) {
        info!("Adding observer {:?}", observer);
        if !can_access_setting(&name, &self.origin_attributes)
            && responder.maybe_send_permission_error(
                &self.origin_attributes,
                "settings:read",
                &format!("add setting observer for {}", name),
            )
        {
            return;
        }

        match self.proxy_tracker.get(&observer) {
            Some(SettingsManagerProxy::SettingObserver(proxy)) => {
                let id = self
                    .state
                    .lock()
                    .db
                    .add_observer(&name, ObserverType::Proxy(proxy.clone()));
                match self.observers.get_mut(&observer) {
                    Some(observer) => {
                        observer.push((name, id));
                    }
                    None => {
                        let init = vec![(name, id)];
                        self.observers.insert(observer, init);
                    }
                }
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked observer");
                responder.reject();
            }
        }
    }

    fn remove_observer(
        &mut self,
        responder: SettingsFactoryRemoveObserverResponder,
        name: String,
        observer: ObjectRef,
    ) {
        if !can_access_setting(&name, &self.origin_attributes)
            && responder.maybe_send_permission_error(
                &self.origin_attributes,
                "settings:read",
                &format!("remove setting observer for {}", name),
            )
        {
            return;
        }

        if self.proxy_tracker.contains_key(&observer) {
            if let Some(target) = self.observers.get_mut(&observer) {
                if let Some(idx) = target.iter().position(|x| x.0 == name) {
                    self.state
                        .lock()
                        .db
                        .remove_observer(&target[idx].0, target[idx].1);
                    target.remove(idx);
                    responder.resolve();
                    return;
                }
            }
            error!("Failed to find observer in list");
        } else {
            error!("Failed to find proxy for this observer");
        }
        responder.reject();
    }
}

common::impl_shared_state!(SettingsService, SettingsSharedData, EmptyConfig);

impl Service<SettingsService> for SettingsService {
    fn create(
        origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<SettingsService, String> {
        info!("SettingsService::create");
        let service_id = helper.session_tracker_id().service();
        let state = Self::shared_state();
        // Require settings:read permission to receive events.
        let dispatcher_id = if origin_attributes.has_permission("settings:read") {
            let event_dispatcher =
                SettingsFactoryEventDispatcher::from(helper, 0 /* object id */);
            Some(state.lock().db.add_dispatcher(&event_dispatcher))
        } else {
            None
        };
        let pool = state.lock().pool.clone();
        Ok(SettingsService {
            id: service_id,
            proxy_tracker: HashMap::new(),
            state,
            dispatcher_id,
            observers: HashMap::new(),
            origin_attributes: origin_attributes.clone(),
            pool,
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<SettingsManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("SettingsService request: {:?}", req),
            Err(err) => format!("Unable to format SettingsService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        self.proxy_tracker.remove(&object_id.into()).is_some()
    }
}

impl Drop for SettingsService {
    fn drop(&mut self) {
        info!("Dropping Settings Service #{}", self.id);
        let db = &mut self.state.lock().db;
        if let Some(dispatcher_id) = self.dispatcher_id {
            db.remove_dispatcher(dispatcher_id);
        }

        // Unregister observers for this instance.
        for observer in self.observers.values() {
            for (name, id) in observer {
                db.remove_observer(name, *id);
            }
        }
    }
}
