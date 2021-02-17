/// A scope based timer.
use std::time::Instant;

pub(crate) struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn start(name: &str) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        log::info!(
            "[timer] {} : {}ms",
            self.name,
            self.start.elapsed().as_millis()
        );
    }
}
