// Simple counters of active sessions, using atomics.

use std::sync::atomic::{AtomicUsize, Ordering};

pub enum SessionKind {
    Ws,
    Uds,
}

impl SessionKind {
    /// Starts an active session of this kind.
    pub fn start(&self) {
        match &self {
            SessionKind::Ws => start_ws_session(),
            SessionKind::Uds => start_uds_session(),
        }
    }

    /// Ends an active session of this kind.
    pub fn end(&self) {
        match &self {
            SessionKind::Ws => end_ws_session(),
            SessionKind::Uds => end_uds_session(),
        }
    }

    /// Returns the number of active session of this kind.
    pub fn count(&self) -> usize {
        match &self {
            SessionKind::Ws => ws_session_count(),
            SessionKind::Uds => uds_session_count(),
        }
    }
}

static WS_SESSIONS: AtomicUsize = AtomicUsize::new(0);
static UDS_SESSIONS: AtomicUsize = AtomicUsize::new(0);

fn ws_session_count() -> usize {
    WS_SESSIONS.load(Ordering::Relaxed)
}

fn start_ws_session() {
    WS_SESSIONS.fetch_add(1, Ordering::Relaxed);
}

fn end_ws_session() {
    WS_SESSIONS.fetch_sub(1, Ordering::Relaxed);
}

fn uds_session_count() -> usize {
    UDS_SESSIONS.load(Ordering::Relaxed)
}

fn start_uds_session() {
    UDS_SESSIONS.fetch_add(1, Ordering::Relaxed);
}

fn end_uds_session() {
    UDS_SESSIONS.fetch_sub(1, Ordering::Relaxed);
}
