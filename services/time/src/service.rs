/// Implementation of the time service.
use crate::generated::common::*;
use crate::generated::service::*;
use crate::time_manager::*;
use common::core::BaseMessage;
use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, TrackerId,
};
use common::SystemTime;
use log::{debug, error, info};
use std::time::SystemTime as StdTime;
use threadpool::ThreadPool;

pub struct Time {
    id: TrackerId,
    pool: ThreadPool,
}

impl TimeService for Time {}

impl TimeMethods for Time {
    fn set(&mut self, responder: &TimeSetResponder, time: SystemTime) {
        let responder = responder.clone();
        self.pool.execute(move || {
            let since_epoch = (*time)
                .duration_since(StdTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                .as_millis();
            match TimeManager::set_system_clock(since_epoch as i64) {
                Ok(success) => {
                    if success {
                        // kernel time is changed, sent event through sidl.
                        // TimeService::time_changed_response(id, sender, &event_map, base);

                        // TODO: broadcast time change event.
                        // Notify web runtime service that time has changed.
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
}

impl Service<Time> for Time {
    type State = ();

    fn shared_state() -> Shared<Self::State> {
        Shared::default()
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        _shared_obj: Shared<Self::State>,
        helper: SessionSupport,
    ) -> Option<Time> {
        info!("TimeoService::create");
        let service_id = helper.session_tracker_id().service();
        let service = Time {
            id: service_id,
            pool: ThreadPool::new(1),
        };

        Some(service)
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
        true
    }
}

impl Drop for Time {
    fn drop(&mut self) {
        debug!("Dropping Time Service #{}", self.id);
    }
}
