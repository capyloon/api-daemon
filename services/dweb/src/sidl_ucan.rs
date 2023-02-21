/// UCAN sidl implementation.
use crate::generated::common::*;
use crate::service::State;
use common::traits::{Shared, SimpleObjectTracker, TrackerId};
use core::str::FromStr;
use ucan::ucan::Ucan;

pub struct SidlUcan {
    inner: Ucan,
    id: TrackerId,
    state: Shared<State>,
}

impl SimpleObjectTracker for SidlUcan {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl SidlUcan {
    pub fn new(id: TrackerId, ucan: Ucan, state: Shared<State>) -> Self {
        Self {
            id,
            inner: ucan,
            state: state.clone(),
        }
    }

    pub fn try_new(id: TrackerId, token: String, state: Shared<State>) -> Option<Self> {
        if let Ok(ucan) = Ucan::from_str(&token) {
            Some(Self::new(id, ucan, state))
        } else {
            None
        }
    }
}

impl UcanMethods for SidlUcan {
    fn encoded(&mut self, responder: UcanEncodedResponder) {
        if let Ok(base64) = self.inner.encode() {
            responder.resolve(base64);
        } else {
            responder.reject(UcanError::InternalError);
        }
    }

    fn remove(&mut self, responder: UcanRemoveResponder) {}

    fn get_blocked(&mut self, responder: UcanGetBlockedResponder) {}

    fn set_blocked(&mut self, value: bool) {}
}
