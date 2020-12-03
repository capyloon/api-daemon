use std::os::raw::{c_char, c_int};
// Link with libpower.so
#[link(name = "power")]
extern "C" {
    pub fn acquire_wake_lock(lock: c_int, id: *const c_char) -> c_int;
    pub fn release_wake_lock(id: *const c_char) -> c_int;
}
