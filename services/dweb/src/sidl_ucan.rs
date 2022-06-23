/// UCAN sidl implementation.
use crate::generated::common::*;
use crate::service::State;
use common::traits::{Shared, SimpleObjectTracker};
use ucan::ucan::Ucan;

pub struct SidlUcan {
    inner: Ucan,
    state: Shared<State>,
}

impl SimpleObjectTracker for SidlUcan {}

impl SidlUcan {
    pub fn new(ucan: Ucan, state: Shared<State>) -> Self {
        Self {
            inner: ucan,
            state: state.clone(),
        }
    }

    pub fn try_new(token: String, state: Shared<State>) -> Option<Self> {
        if let Ok(ucan) = Ucan::try_from_token_string(&token) {
            Some(Self::new(ucan, state))
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
