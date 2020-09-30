use crate::generated::ffi::*;
use std::ptr::null_mut;
use crate::signal_context::SignalContext;

// Wrapper around the Store context.
pub type StoreContextPtr = *mut signal_protocol_store_context;

pub struct StoreContext {
    native: StoreContextPtr,
}

impl StoreContext {
    // Creates a new store context, or return None if that failed.
    pub fn new(global_context: &SignalContext) -> Option<Self> {
        let mut ctxt: StoreContextPtr = null_mut();
        if unsafe { signal_protocol_store_context_create(&mut ctxt, global_context.native()) } == 0
        {
            return Some(StoreContext { native: ctxt });
        } else {
            debug!("Error in signal_context_create");
        }

        None
    }

    pub fn native(&self) -> StoreContextPtr {
        self.native
    }

    pub fn set_session_store(&self, store: signal_protocol_session_store) -> bool {
        (unsafe { signal_protocol_store_context_set_session_store(self.native, &store) } == 0)
    }

    pub fn set_pre_key_store(&self, store: signal_protocol_pre_key_store) -> bool {
        (unsafe { signal_protocol_store_context_set_pre_key_store(self.native, &store) } == 0)
    }

    pub fn set_signed_pre_key_store(&self, store: signal_protocol_signed_pre_key_store) -> bool {
        (unsafe { signal_protocol_store_context_set_signed_pre_key_store(self.native, &store) }
            == 0)
    }

    pub fn set_identity_key_store(&self, store: signal_protocol_identity_key_store) -> bool {
        (unsafe { signal_protocol_store_context_set_identity_key_store(self.native, &store) } == 0)
    }

    pub fn set_sender_key_store(&self, store: signal_protocol_sender_key_store) -> bool {
        (unsafe { signal_protocol_store_context_set_sender_key_store(self.native, &store) } == 0)
    }
}

impl Drop for StoreContext {
    fn drop(&mut self) {
        debug!("Dropping StoreContext");
        unsafe {
            signal_protocol_store_context_destroy(self.native);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::signal_context::SignalContext;

    #[test]
    fn create_store_context() {
        let s_context = SignalContext::new().unwrap();
        let context = StoreContext::new(&s_context);
        assert!(context.is_some());
    }
}
