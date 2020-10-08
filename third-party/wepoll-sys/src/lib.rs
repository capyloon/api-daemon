#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! A unsafe/raw wrapper around wepoll.
//!
//! The bindings in this crate were originally generated using bindgen, then
//! cleaned up by hand. They are maintained manually to reduce the amount of
//! dependencies and build-time complexity. As wepoll's public API rarely
//! changes, this should not pose any problems.
use std::os::raw::{c_int, c_void};

pub const EPOLLIN: u32 = 1;
pub const EPOLLPRI: u32 = 2;
pub const EPOLLOUT: u32 = 4;
pub const EPOLLERR: u32 = 8;
pub const EPOLLHUP: u32 = 16;
pub const EPOLLRDNORM: u32 = 64;
pub const EPOLLRDBAND: u32 = 128;
pub const EPOLLWRNORM: u32 = 256;
pub const EPOLLWRBAND: u32 = 512;
pub const EPOLLMSG: u32 = 1024;
pub const EPOLLRDHUP: u32 = 8192;
pub const EPOLLONESHOT: u32 = 2147483648;
pub const EPOLL_CTL_ADD: u32 = 1;
pub const EPOLL_CTL_MOD: u32 = 2;
pub const EPOLL_CTL_DEL: u32 = 3;

pub const EPOLL_EVENTS_EPOLLIN: EPOLL_EVENTS = 1;
pub const EPOLL_EVENTS_EPOLLPRI: EPOLL_EVENTS = 2;
pub const EPOLL_EVENTS_EPOLLOUT: EPOLL_EVENTS = 4;
pub const EPOLL_EVENTS_EPOLLERR: EPOLL_EVENTS = 8;
pub const EPOLL_EVENTS_EPOLLHUP: EPOLL_EVENTS = 16;
pub const EPOLL_EVENTS_EPOLLRDNORM: EPOLL_EVENTS = 64;
pub const EPOLL_EVENTS_EPOLLRDBAND: EPOLL_EVENTS = 128;
pub const EPOLL_EVENTS_EPOLLWRNORM: EPOLL_EVENTS = 256;
pub const EPOLL_EVENTS_EPOLLWRBAND: EPOLL_EVENTS = 512;
pub const EPOLL_EVENTS_EPOLLMSG: EPOLL_EVENTS = 1024;
pub const EPOLL_EVENTS_EPOLLRDHUP: EPOLL_EVENTS = 8192;
pub const EPOLL_EVENTS_EPOLLONESHOT: EPOLL_EVENTS = -2147483648;
pub type EPOLL_EVENTS = i32;

pub type HANDLE = *mut c_void;
pub type SOCKET = usize;

#[repr(C)]
#[derive(Copy, Clone)]
pub union epoll_data {
    pub ptr: *mut c_void,
    pub fd: c_int,
    pub u32: u32,
    pub u64: u64,
    pub sock: SOCKET,
    pub hnd: HANDLE,
    _bindgen_union_align: u64,
}

pub type epoll_data_t = epoll_data;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct epoll_event {
    pub events: u32,
    pub data: epoll_data_t,
}

extern "C" {
    pub fn epoll_create(size: c_int) -> HANDLE;
    pub fn epoll_create1(flags: c_int) -> HANDLE;
    pub fn epoll_close(ephnd: HANDLE) -> c_int;

    pub fn epoll_ctl(
        ephnd: HANDLE,
        op: c_int,
        sock: SOCKET,
        event: *mut epoll_event,
    ) -> c_int;

    pub fn epoll_wait(
        ephnd: HANDLE,
        events: *mut epoll_event,
        maxevents: c_int,
        timeout: c_int,
    ) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::net::UdpSocket;
    use std::os::windows::io::AsRawSocket;

    #[test]
    fn test_poll() {
        unsafe {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .expect("Failed to bind the UDP socket");

            let epoll = epoll_create(1);

            if epoll.is_null() {
                panic!("epoll_create(1) failed");
            }

            let mut event = epoll_event {
                events: EPOLLOUT | EPOLLONESHOT,
                data: epoll_data { u64: 42 },
            };

            epoll_ctl(
                epoll,
                EPOLL_CTL_ADD as i32,
                socket.as_raw_socket() as usize,
                &mut event as *mut _,
            );

            let mut events: [epoll_event; 1] =
                mem::MaybeUninit::uninit().assume_init();
            let received = epoll_wait(epoll, events.as_mut_ptr(), 1, -1);

            epoll_close(epoll);

            assert_eq!(received, 1);
            assert_eq!(events[0].data.u64, 42);
        }
    }
}
