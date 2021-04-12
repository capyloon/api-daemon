/// Implementation of the time service.
use crate::generated::common::*;
use crate::generated::service::*;
use crate::time_manager::*;
use android_utils::{AndroidProperties, PropertyGetter};
use common::core::BaseMessage;
use common::observers::{ObserverTracker, ServiceObserverTracker};
use common::traits::{
    CommonResponder, DispatcherId, OriginAttributes, Service, SessionSupport, Shared,
    SharedSessionContext, StateLogger, TrackerId,
};
use common::{JsonValue, SystemTime};
use log::{debug, error, info};
use settings_service::db::{DbObserver, ObserverType};
use settings_service::service::SettingsService;
use std::collections::HashMap;
use std::time::SystemTime as StdTime;
use threadpool::ThreadPool;

#[derive(Default)]
pub struct SharedObj {
    event_broadcaster: TimeEventBroadcaster,
    // An observer tracker, using the callback reason as key.
    observers: ObserverTracker<CallbackReason, TimeObserverProxy>,
}

impl StateLogger for SharedObj {
    fn log(&self) {
        info!(
            "  {} registered observers ({} keys)",
            self.observers.count(),
            self.observers.key_count()
        );

        self.event_broadcaster.log();
    }
}

impl SharedObj {
    pub fn default() -> Self {
        let setting_service = SettingsService::shared_state();

        // the life time of SharedObj is the same as the process. We don't need to remove_observer
        let id = setting_service.lock().db.add_observer(
            "time.timezone",
            ObserverType::FuncPtr(Box::new(SettingObserver {})),
        );

        info!("add_observer to SettingsService with id {}", id);
        SharedObj {
            ..Default::default()
        }
    }

    pub fn add_observer(
        &mut self,
        reason: CallbackReason,
        observer: &TimeObserverProxy,
    ) -> DispatcherId {
        self.observers.add(reason, observer.clone())
    }

    pub fn remove_observer(&mut self, reason: CallbackReason, id: DispatcherId) -> bool {
        self.observers.remove(&reason, id)
    }

    pub fn broadcast(&mut self, rn: CallbackReason, tz: String, time_delta: i64) {
        let mut info = TimeInfo {
            reason: rn,
            timezone: tz,
            delta: time_delta,
        };

        if info.timezone.is_empty() {
            // caller doesn't specify timezone, get local timezone setting
            if let Ok(tz) = AndroidProperties::get("persist.sys.timezone", "") {
                info.timezone = tz;
            }
        }

        self.observers.for_each(&info.reason, |proxy, _id| {
            proxy.callback(info.clone());
        });

        match info.reason {
            CallbackReason::TimeChanged => self.event_broadcaster.broadcast_time_changed(),
            CallbackReason::TimezoneChanged => self.event_broadcaster.broadcast_timezone_changed(),
            CallbackReason::None => error!("unexpected callback reason {:?}", info.reason),
        }
    }
}

lazy_static! {
    pub(crate) static ref TIME_SHARED_DATA: Shared<SharedObj> = Shared::adopt(SharedObj::default());
}

#[derive(Clone, Copy)]
struct SettingObserver {}

impl DbObserver for SettingObserver {
    fn callback(&self, name: &str, value: &JsonValue) {
        if name != "time.timezone" {
            error!(
                "unexpected key {} / value {}",
                name,
                value.as_str().unwrap().to_string()
            );
            return;
        }

        let timezone = value.as_str().unwrap().to_string();
        match TimeManager::set_timezone(timezone.clone()) {
            Ok(_) => {
                let shared = Time::shared_state();
                let mut shared_lock = shared.lock();
                shared_lock.broadcast(CallbackReason::TimezoneChanged, timezone, 0);
            }
            Err(e) => error!("set timezone failed: {:?}", e),
        }
    }
}

pub struct Time {
    id: TrackerId,
    pool: ThreadPool,
    shared_obj: Shared<SharedObj>,
    dispatcher_id: DispatcherId,
    proxy_tracker: TimeServiceProxyTracker,
    observers: ServiceObserverTracker<CallbackReason>,
    origin_attributes: OriginAttributes,
}

impl TimeService for Time {
    fn get_proxy_tracker(&mut self) -> &mut TimeServiceProxyTracker {
        &mut self.proxy_tracker
    }
}

impl TimeMethods for Time {
    fn set(&mut self, responder: &TimeSetResponder, time: SystemTime) {
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "system-time:write",
            "set system time",
        ) {
            return;
        }

        let responder = responder.clone();
        self.pool.execute(move || {
            let since_epoch = (*time)
                .duration_since(StdTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                .as_millis();

            // get time difference
            let mut time_delta = 0;
            match TimeManager::get_system_clock() {
                Ok(cur) => {
                    time_delta = (since_epoch as i64) - cur;
                }
                Err(e) => {
                    error!("get time failed {:?}", e);
                }
            }

            match TimeManager::set_system_clock(since_epoch as i64) {
                Ok(success) => {
                    if success {
                        let shared_obj = Time::shared_state();
                        let mut shared_lock = shared_obj.lock();
                        info!("broadcast time changed event ");
                        shared_lock.broadcast(
                            CallbackReason::TimeChanged,
                            "".to_string(),
                            time_delta,
                        );
                        responder.resolve();
                    } else {
                        responder.reject();
                    }
                }
                Err(e) => {
                    responder.reject();
                    error!("set time failed:{:?}", e);
                }
            }
        });
    }

    fn get(&mut self, responder: &TimeGetResponder) {
        match TimeManager::get_system_clock() {
            Ok(since_epoch) => {
                let time = StdTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(since_epoch as u64))
                    .unwrap_or(StdTime::UNIX_EPOCH);
                responder.resolve(SystemTime::from(time));
            }
            Err(e) => {
                error!("get time failed: {:?}", e);
                responder.reject()
            }
        }
    }

    fn set_timezone(&mut self, responder: &TimeSetTimezoneResponder, timezone: String) {
        info!("set time zone {:?}", timezone);
        if responder.maybe_send_permission_error(
            &self.origin_attributes,
            "system-time:write",
            "set system timezone",
        ) {
            return;
        }
        match TimeManager::set_timezone(timezone.clone()) {
            Ok(_) => {
                let mut shared_lock = self.shared_obj.lock();
                info!("broadcast timezone changed event");
                shared_lock.broadcast(CallbackReason::TimezoneChanged, timezone, 0);
                responder.resolve();
            }
            Err(e) => {
                error!("set timezone failed:{:?}", e);
                responder.reject();
            }
        }
    }

    fn get_elapsed_real_time(&mut self, responder: &TimeGetElapsedRealTimeResponder) {
        info!("get_elapse_real_time");
        match TimeManager::get_elapsed_real_time() {
            Ok(success) => responder.resolve(success),
            Err(_) => responder.reject(),
        }
    }

    fn add_observer(
        &mut self,
        responder: &TimeAddObserverResponder,
        reason: CallbackReason,
        observer: ObjectRef,
    ) {
        info!("Adding observer {:?}", observer);

        match self.proxy_tracker.get(&observer) {
            Some(TimeServiceProxy::TimeObserver(proxy)) => {
                let id = self.shared_obj.lock().add_observer(reason, proxy);
                self.observers.add(observer.into(), reason, id);
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
        responder: &TimeRemoveObserverResponder,
        reason: CallbackReason,
        observer: ObjectRef,
    ) {
        info!("Remove observer {:?}", observer);

        if self.proxy_tracker.contains_key(&observer) {
            let shared_lock = &mut self.shared_obj.lock();
            if self
                .observers
                .remove(observer.into(), reason, &mut shared_lock.observers)
            {
                responder.resolve();
            } else {
                error!("Failed to find observer in list");
                responder.reject();
            }
        } else {
            error!("Failed to find proxy for this observer");
            responder.reject();
        }
    }
}

impl Service<Time> for Time {
    type State = SharedObj;

    fn shared_state() -> Shared<Self::State> {
        let shared = &*TIME_SHARED_DATA;
        shared.clone()
    }

    fn create(
        origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        _shared_obj: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Result<Time, String> {
        info!("TimeService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = TimeEventDispatcher::from(helper, 0);
        let dispatcher_id = _shared_obj.lock().event_broadcaster.add(&event_dispatcher);

        Ok(Time {
            id: service_id,
            pool: ThreadPool::new(1),
            shared_obj: _shared_obj,
            dispatcher_id,
            proxy_tracker: HashMap::new(),
            observers: ServiceObserverTracker::default(),
            origin_attributes: origin_attributes.clone(),
        })
    }

    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        info!("TimeManager::format_request");
        let req: Result<TimeServiceFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("TimeManager service request: {:?}", req),
            Err(err) => format!("Unable to TimeManager service request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        info!("incoming request");
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        info!("releasing object {}", object_id);
        self.proxy_tracker.remove(&object_id.into()).is_some()
    }
}

impl Drop for Time {
    fn drop(&mut self) {
        debug!(
            "Dropping Time Service#{}, dispatcher_id {}",
            self.id, self.dispatcher_id
        );
        let shared_lock = &mut self.shared_obj.lock();
        shared_lock.event_broadcaster.remove(self.dispatcher_id);
        self.observers.clear(&mut shared_lock.observers);
    }
}
