use crate::generated::common::{
    Address, IdentityKeyStoreProxy, KeyStoreProxy, SenderKeyName, SenderKeyStoreProxy,
    SessionStoreProxy,
};
use libsignal_sys::ffi::*;
use libsignal_sys::{SignalContext as FfiSignalContext, StoreContext as FfiStoreContext};
use log::{debug, error};
use std::os::raw::{c_char, c_int, c_void};
use std::rc::Rc;
use std::sync::mpsc::Receiver;

type BufferPtr = *mut signal_buffer;

static SIGNAL_GENERIC_ERROR: c_int = -1;
static SIGNAL_SUCCESS: c_int = 0;
static SIGNAL_ERR_INVALID_KEY_ID: c_int = -1003;

// This structs are used to bridge the native signal store with our implementation.

#[derive(Clone)]
pub struct StoreProxies {
    pub session_store: SessionStoreProxy,
    pub identity_key_store: IdentityKeyStoreProxy,
    pub pre_key_store: KeyStoreProxy,
    pub signed_pre_key_store: KeyStoreProxy,
    pub sender_key_store: SenderKeyStoreProxy,
}

impl Drop for StoreProxies {
    fn drop(&mut self) {
        error!("StoreProxies::drop");
    }
}

pub struct StoreContext {
    pub ffi: FfiStoreContext,
    pub proxies: Rc<StoreProxies>,
}

impl StoreContext {
    pub fn new(global_ctxt: &FfiSignalContext, proxies: Rc<StoreProxies>) -> Option<Self> {
        if let Some(ffi) = FfiStoreContext::new(global_ctxt) {
            // Leak temporarily to use the raw address in the native callbacks.
            let raw = Rc::into_raw(proxies);

            // Setup the session store.
            let session_store = signal_protocol_session_store {
                load_session_func: Some(session_store_load_session),
                get_sub_device_sessions_func: Some(session_store_get_sub_device_sessions),
                store_session_func: Some(session_store_store_session),
                contains_session_func: Some(session_store_contains_session),
                delete_session_func: Some(session_store_delete_session),
                delete_all_sessions_func: Some(session_store_delete_all_sessions),
                destroy_func: Some(session_store_destroy),
                user_data: raw as _,
            };

            // Setup the identity store.
            let identity_store = signal_protocol_identity_key_store {
                get_identity_key_pair: Some(id_key_store_get_identity_key_pair),
                get_local_registration_id: Some(id_key_store_get_local_registration_id),
                save_identity: Some(id_key_store_save_identity),
                is_trusted_identity: Some(id_key_store_is_trusted_identity),
                destroy_func: Some(id_key_store_destroy),
                user_data: raw as _,
            };

            // Setup the sender store.
            let sender_store = signal_protocol_sender_key_store {
                store_sender_key: Some(store_sender_key),
                load_sender_key: Some(load_sender_key),
                destroy_func: Some(sender_key_store_destroy),
                user_data: raw as _,
            };

            // Setup the prekey store.
            let pre_key_store = signal_protocol_pre_key_store {
                load_pre_key: Some(load_pre_key),
                store_pre_key: None,
                contains_pre_key: Some(contains_pre_key),
                remove_pre_key: Some(remove_pre_key),
                destroy_func: Some(pre_key_store_destroy),
                user_data: raw as _,
            };

            // Setup the signed prekey store.
            let signed_pre_key_store = signal_protocol_signed_pre_key_store {
                load_signed_pre_key: Some(load_signed_pre_key),
                store_signed_pre_key: None,
                contains_signed_pre_key: Some(contains_signed_pre_key),
                remove_signed_pre_key: Some(remove_signed_pre_key),
                destroy_func: Some(signed_pre_key_store_destroy),
                user_data: raw as _,
            };

            // Make sure we won't leak.
            let proxies: Rc<StoreProxies> = unsafe { Rc::from_raw(raw) };

            if !ffi.set_session_store(session_store)
                || !ffi.set_identity_key_store(identity_store)
                || !ffi.set_sender_key_store(sender_store)
                || !ffi.set_pre_key_store(pre_key_store)
                || !ffi.set_signed_pre_key_store(signed_pre_key_store)
            {
                return None;
            }

            return Some(StoreContext { ffi, proxies });
        }
        None
    }

    pub fn ffi(&self) -> &FfiStoreContext {
        &self.ffi
    }
}

impl Drop for StoreContext {
    fn drop(&mut self) {
        error!("StoreContext::drop");
    }
}

// Session store functions.

/// @return 1 if the session was loaded, 0 if the session was not found, negative on failure
extern "C" fn session_store_load_session(
    record: *mut *mut signal_buffer,
    _user_record: *mut *mut signal_buffer,
    address: *const signal_protocol_address,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("session_store_load_session");

    // 1. Call session_store.load(address) using the proxy.
    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };

    let receiver = Rc::make_mut(&mut ctxt).session_store.load(addr);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    // 2. Wait for the answer and return the response.
    build_response(receiver, "session_store.load", |buffer| {
        if let Some(buffer) = buffer {
            // If the record is empty, that means "not found".
            if buffer.is_empty() {
                0
            } else {
                // Put the result in the record.
                unsafe {
                    *record = signal_buffer::from_slice(&buffer);
                }
                1
            }
        } else {
            0
        }
    })
}

/// @return size of the sessions array, or negative on failure
extern "C" fn session_store_get_sub_device_sessions(
    sessions: *mut *mut signal_int_list,
    name: *const c_char,
    name_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    error!("session_store_get_sub_device_sessions");

    let session_name =
        unsafe { String::from_raw_parts(name as *mut u8, name_len as _, name_len as _) };
    let name = session_name.clone();

    // The session_name lifetime is managed by signal, so forget it.
    ::std::mem::forget(session_name);

    let receiver = Rc::make_mut(&mut ctxt)
        .session_store
        .get_sub_device_sessions(name);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(
        receiver,
        "session_store.get_sub_device_sessions",
        |response| {
            let list = response.unwrap_or_default();
            // list is a Vec<> of ids.
            // Build a signal_int_list from that vector.

            let size = list.len() as c_int;
            unsafe {
                *sessions =
                    int_list_from_vec(list.into_iter().map(|value| value as c_int).collect())
            }
            size
        },
    )
}

/// @return 0 on success, negative on failure
extern "C" fn session_store_store_session(
    address: *const signal_protocol_address,
    record: *mut u8,
    record_len: size_t,
    _user_record: *mut u8,
    _user_record_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("session_store_store_session");

    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };

    let vec = unsafe { Vec::from_raw_parts(record, record_len as _, record_len as _) };
    let data = vec.clone();

    // libsignal manages the lifetime of this array.
    ::std::mem::forget(vec);

    let receiver = Rc::make_mut(&mut ctxt).session_store.store(addr, data);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "session_store.store", |_| SIGNAL_SUCCESS)
}

/// @return 1 if a session record exists, 0 otherwise.
extern "C" fn session_store_contains_session(
    address: *const signal_protocol_address,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("session_store_contains_session");

    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };

    let receiver = Rc::make_mut(&mut ctxt).session_store.contains(addr);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response_with_error(receiver, 0, "session_store.contains", |response| {
        if response {
            1
        } else {
            0
        }
    })
}

/// @return 1 if a session was deleted, 0 if a session was not deleted, negative on error
extern "C" fn session_store_delete_session(
    address: *const signal_protocol_address,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("session_store_delete_session");

    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };

    let receiver = Rc::make_mut(&mut ctxt).session_store.delete(addr);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "session_store.delete", |response| {
        if response {
            1
        } else {
            0
        }
    })
}

/// @return the number of deleted sessions on success, negative on failure
extern "C" fn session_store_delete_all_sessions(
    name: *const c_char,
    name_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    error!("session_store_delete_all_sessions");

    let target_name =
        unsafe { String::from_raw_parts(name as *mut u8, name_len as _, name_len as _) };

    let name = target_name.clone();

    // The name lifetime is managed by signal, so forget it.
    ::std::mem::forget(target_name);

    let receiver = Rc::make_mut(&mut ctxt)
        .session_store
        .delete_all_sessions(name);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "session_store.delete_all_sessions", |response| {
        response as _
    })
}

extern "C" fn session_store_destroy(_user_data: *mut c_void) {
    // We don't deref user_data here because the lifetime of the StoreContext
    // is managed by Rust.
    debug!("session_store_destroy");
}

// Identity key store functions

/// @return 0 on success, negative on failure
extern "C" fn id_key_store_get_identity_key_pair(
    public_data: *mut BufferPtr,
    private_data: *mut BufferPtr,
    user_data: *mut c_void,
) -> c_int {
    debug!(
        "id_key_store_get_identity_key_pair {:?} {:?}",
        public_data, private_data
    );
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt).identity_key_store.get_key_pair();

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "id_key_store.get_key_pair", |key_pair| {
        // get key pair...
        debug!("Got keypair {:?}", key_pair);
        unsafe {
            *public_data = signal_buffer::from_public_key(&key_pair.public_key);
            *private_data = signal_buffer::from_slice(&key_pair.private_key);
        }
        SIGNAL_SUCCESS
    })
}

/// @return 0 on success, negative on failure
extern "C" fn id_key_store_get_local_registration_id(
    user_data: *mut c_void,
    registration_id: *mut u32,
) -> c_int {
    debug!("id_key_store_get_local_registration_id");
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt)
        .identity_key_store
        .get_local_registration_id();

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(
        receiver,
        "id_key_store.get_local_registration_id",
        |result| {
            unsafe {
                *registration_id = result as u32;
            }
            SIGNAL_SUCCESS
        },
    )
}

/// @return 0 on success, negative on failure
extern "C" fn id_key_store_save_identity(
    address: *const signal_protocol_address,
    key_data: *mut u8,
    key_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    debug!("id_key_store_save_identity");
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };
    let key = unsafe { Vec::from_raw_parts(key_data, key_len as _, key_len as _) };
    let data = key.clone();

    // The key lifetime is managed by signal, so forget it.
    ::std::mem::forget(key);

    let receiver = Rc::make_mut(&mut ctxt)
        .identity_key_store
        .save_identity(addr, data);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "id_key_store.save_identity", |_| SIGNAL_SUCCESS)
}

/// @return 1 if trusted, 0 if untrusted, negative on failure
extern "C" fn id_key_store_is_trusted_identity(
    address: *const signal_protocol_address,
    key_data: *mut u8,
    key_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    debug!("id_key_store_is_trusted_identity {}", unsafe {
        (*address).to_string()
    });
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let addr = Address {
        name: unsafe { (*address).to_string() },
        device_id: unsafe { (*address).device_id } as i64,
    };
    let key = unsafe { Vec::from_raw_parts(key_data, key_len as _, key_len as _) };
    let data = key.clone();

    // The key lifetime is managed by signal, so forget it.
    ::std::mem::forget(key);

    let receiver = Rc::make_mut(&mut ctxt)
        .identity_key_store
        .is_trusted_identity(addr, data);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "id_key_store.is_truster_identity", |result| {
        if result {
            1
        } else {
            0
        }
    })
}

extern "C" fn id_key_store_destroy(_user_data: *mut c_void) {
    // We don't deref user_data here because the lifetime of the StoreContext
    // is managed by Rust.
    debug!("id_key_store_destroy");
}

// Sender Key store functions.

/// @param sender_key_name the (groupId + senderId + deviceId) tuple
/// @param record pointer to a buffer containing the serialized record
/// @param record_len length of the serialized record
/// @return 0 on success, negative on failure
extern "C" fn store_sender_key(
    sender_key_name: *const signal_protocol_sender_key_name,
    record: *mut u8,
    record_len: size_t,
    _user_record: *mut u8,
    _user_record_len: size_t,
    user_data: *mut c_void,
) -> c_int {
    debug!("store_sender_key");
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let group_id = unsafe {
        String::from_raw_parts(
            (*sender_key_name).group_id as *mut u8,
            (*sender_key_name).group_id_len as _,
            (*sender_key_name).group_id_len as _,
        )
    };

    let name = unsafe {
        String::from_raw_parts(
            (*sender_key_name).sender.name as *mut u8,
            (*sender_key_name).sender.name_len as _,
            (*sender_key_name).sender.name_len as _,
        )
    };

    let sender = SenderKeyName {
        group_id: group_id.clone(),
        sender: Address {
            name: name.clone(),
            device_id: unsafe { (*sender_key_name).sender.device_id.into() },
        },
    };

    let vec = unsafe { Vec::from_raw_parts(record, record_len as _, record_len as _) };
    let data = vec.clone();

    // These lifetimes are managed by signal, so forget them.
    ::std::mem::forget(group_id);
    ::std::mem::forget(name);
    ::std::mem::forget(vec);

    let receiver = Rc::make_mut(&mut ctxt).sender_key_store.store(sender, data);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "sender_key_store.store_sender_key", |result| {
        if result {
            0
        } else {
            SIGNAL_GENERIC_ERROR
        }
    })
}

/// Returns a copy of the sender key record corresponding to the
/// (groupId + senderId + deviceId) tuple.
///
/// @param record pointer to a newly allocated buffer containing the record,
/// if found. Unset if no record was found.
/// The Signal Protocol library is responsible for freeing this buffer.
/// @param sender_key_name the (groupId + senderId + deviceId) tuple
/// @return 1 if the record was loaded, 0 if the record was not found, negative on failure
extern "C" fn load_sender_key(
    record: *mut *mut signal_buffer,
    _user_record: *mut *mut signal_buffer,
    sender_key_name: *const signal_protocol_sender_key_name,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let group_id = unsafe {
        String::from_raw_parts(
            (*sender_key_name).group_id as *mut u8,
            (*sender_key_name).group_id_len as _,
            (*sender_key_name).group_id_len as _,
        )
    };

    let name = unsafe {
        String::from_raw_parts(
            (*sender_key_name).sender.name as *mut u8,
            (*sender_key_name).sender.name_len as _,
            (*sender_key_name).sender.name_len as _,
        )
    };

    let sender = SenderKeyName {
        group_id: group_id.clone(),
        sender: Address {
            name: name.clone(),
            device_id: unsafe { (*sender_key_name).sender.device_id.into() },
        },
    };

    // These lifetimes are managed by libsignal.
    ::std::mem::forget(group_id);
    ::std::mem::forget(name);

    let receiver = Rc::make_mut(&mut ctxt).sender_key_store.load(sender);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "sender_key_store.load_sender_key", |buffer| {
        if let Some(buffer) = buffer {
            // If the record is empty, that means "not found".
            if buffer.is_empty() {
                0
            } else {
                // Put the result in the record.
                unsafe {
                    *record = signal_buffer::from_slice(&buffer);
                }
                1
            }
        } else {
            0
        }
    })
}

/// Function called to perform cleanup when the data store context is being
/// destroyed.
extern "C" fn sender_key_store_destroy(_user_data: *mut c_void) {
    // We don't deref user_data here because the lifetime of the StoreContext
    // is managed by Rust.
    debug!("sender_key_store_destroy");
}

// Pre key store functions.
// TODO: Find a way to share with the signed pre key ones.

/// @retval SG_SUCCESS if the key was found
/// @retval SG_ERR_INVALID_KEY_ID if the key could not be found
extern "C" fn load_pre_key(
    record: *mut SignalBufferPtr,
    pre_key_id: u32,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("load_pre_key");

    let receiver = Rc::make_mut(&mut ctxt).pre_key_store.load(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "pre_key_store.load", |buffer| {
        if let Some(buffer) = buffer {
            // If the record is empty, that means "not found".
            if buffer.is_empty() {
                SIGNAL_ERR_INVALID_KEY_ID
            } else {
                // Put the result in the record.
                unsafe {
                    *record = signal_buffer::from_slice(&buffer);
                }
                SIGNAL_SUCCESS
            }
        } else {
            SIGNAL_ERR_INVALID_KEY_ID
        }
    })
}

/// @return 1 if the store has a record for the PreKey ID, 0 otherwise
extern "C" fn contains_pre_key(pre_key_id: u32, user_data: *mut c_void) -> c_int {
    debug!("contains_pre_key id={}", pre_key_id);
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt)
        .pre_key_store
        .contains(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response_with_error(receiver, 0, "pre_key_store.contains", |result| {
        if result {
            1
        } else {
            0
        }
    })
}

/// @return 0 on success, negative on failure
extern "C" fn remove_pre_key(pre_key_id: u32, user_data: *mut c_void) -> c_int {
    debug!("remove_pre_key id={}", pre_key_id);
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt)
        .pre_key_store
        .remove(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "pre_key_store.remove", |_| SIGNAL_SUCCESS)
}

extern "C" fn pre_key_store_destroy(_user_data: *mut c_void) {
    // We don't deref user_data here because the lifetime of the StoreContext
    // is managed by Rust.
    debug!("pre_key_store_destroy");
}

// Signed Pre key store functions.
// TODO: Find a way to share with the pre key ones.

/// @retval SG_SUCCESS if the key was found
/// @retval SG_ERR_INVALID_KEY_ID if the key could not be found
extern "C" fn load_signed_pre_key(
    record: *mut SignalBufferPtr,
    pre_key_id: u32,
    user_data: *mut c_void,
) -> c_int {
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };
    debug!("load_signed_pre_key");

    let receiver = Rc::make_mut(&mut ctxt)
        .signed_pre_key_store
        .load(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "signed_pre_key_store.load", |buffer| {
        if let Some(buffer) = buffer {
            // If the record is empty, that means "not found".
            if buffer.is_empty() {
                SIGNAL_ERR_INVALID_KEY_ID
            } else {
                // Put the result in the record.
                unsafe {
                    *record = signal_buffer::from_slice(&buffer);
                }
                SIGNAL_SUCCESS
            }
        } else {
            SIGNAL_ERR_INVALID_KEY_ID
        }
    })
}

/// @return 1 if the store has a record for the PreKey ID, 0 otherwise
extern "C" fn contains_signed_pre_key(pre_key_id: u32, user_data: *mut c_void) -> c_int {
    debug!("contains_signed_pre_key id={}", pre_key_id);
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt)
        .signed_pre_key_store
        .contains(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response_with_error(receiver, 0, "signed_pre_key_store.contains", |result| {
        if result {
            1
        } else {
            0
        }
    })
}

/// @return 0 on success, negative on failure
extern "C" fn remove_signed_pre_key(pre_key_id: u32, user_data: *mut c_void) -> c_int {
    debug!("remove_signed_pre_key id={}", pre_key_id);
    let mut ctxt: Rc<StoreProxies> = unsafe { Rc::from_raw(user_data as *const StoreProxies) };

    let receiver = Rc::make_mut(&mut ctxt)
        .signed_pre_key_store
        .remove(pre_key_id as _);

    // Make sure we'll release ownership.
    let _ = Rc::into_raw(ctxt);

    build_response(receiver, "signed_pre_key_store.remove", |_| SIGNAL_SUCCESS)
}

extern "C" fn signed_pre_key_store_destroy(_user_data: *mut c_void) {
    // We don't deref user_data here because the lifetime of the StoreContext
    // is managed by Rust.
    debug!("pre_key_store_destroy");
}

/// Helper functions to avoid boilerplate.
fn build_response_with_error<T, E, F>(
    receiver: Receiver<Result<T, E>>,
    default_error: c_int,
    fn_name: &str,
    closure: F,
) -> c_int
where
    E: std::fmt::Debug,
    F: FnOnce(T) -> c_int,
{
    match receiver.recv() {
        Ok(Ok(response)) => closure(response),
        Ok(Err(err)) => {
            error!("Error returned from {}: {:?}", fn_name, err);
            default_error
        }
        Err(err) => {
            error!("Error receiving from {}: {}", fn_name, err);
            default_error
        }
    }
}

fn build_response<T, E, F>(receiver: Receiver<Result<T, E>>, fn_name: &str, closure: F) -> c_int
where
    E: std::fmt::Debug,
    F: FnOnce(T) -> c_int,
{
    build_response_with_error(receiver, SIGNAL_GENERIC_ERROR, fn_name, closure)
}
