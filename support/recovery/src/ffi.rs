use std::os::raw::{c_char, c_int};
// Link with librecovery.so
#[link(name = "recovery")]
extern "C" {
    pub fn factoryReset() -> c_int;
    pub fn installFotaUpdate(updatePath: *const c_char, updatePathLength: c_int) -> c_int;
}
