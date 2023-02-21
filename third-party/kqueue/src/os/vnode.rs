use kqueue_sys::constants::FilterFlag;

use super::super::Vnode;

#[cfg(target_os = "freebsd")]
pub(crate) fn handle_vnode_extras(ff: FilterFlag) -> Vnode {
    if ff.contains(FilterFlag::NOTE_CLOSE_WRITE) {
        Vnode::CloseWrite
    } else if ff.contains(FilterFlag::NOTE_CLOSE) {
        Vnode::Close
    } else if ff.contains(FilterFlag::NOTE_OPEN) {
        Vnode::Open
    } else {
        panic!("not supported")
    }
}

#[cfg(not(target_os = "freebsd"))]
pub(crate) fn handle_vnode_extras(_ff: FilterFlag) -> Vnode {
    panic!("not supported")
}
