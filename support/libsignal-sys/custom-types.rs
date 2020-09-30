use std::ffi::CString;
use std::os::raw::c_int;
#[cfg(test)]
use crate::generated::libsignal_proto::*;
#[cfg(test)]
use prost::Message;

const DJB_KEY_LEN: usize = 32;
const DJB_TYPE: u8 = 0x05;

// Builds a key from an array.
macro_rules! build_key {
    ($name:ident, $kind:ident, $from:expr) => {
        let mut $name = $kind::default();
        $name.data = *array_ref!($from, 0, 32);
        $name.base.ref_count = 1;
        $name.base.destroy = None;
    };
}

pub type KeyArray = [u8; DJB_KEY_LEN];

pub type SignalBufferPtr = *mut signal_buffer;

#[repr(C)]
pub struct signal_buffer {
    pub len: usize,
    pub data: [u8; 0],
}

impl signal_buffer {
    pub fn data_slice(&mut self) -> &mut [u8] {
        unsafe { ::std::slice::from_raw_parts_mut(self.data.as_mut_ptr(), self.len as usize) }
    }

    pub fn from_slice(data: &[u8]) -> *mut signal_buffer {
        unsafe { signal_buffer_create(data.as_ptr(), data.len()) }
    }

    pub fn from_public_key(data: &[u8]) -> *mut signal_buffer {
        let mut v = vec![DJB_TYPE];
        v.extend_from_slice(data);
        signal_buffer::from_slice(&v)
    }

    pub fn from_private_key(data: &[u8]) -> *mut signal_buffer {
        signal_buffer::from_slice(data)
    }
}

#[repr(C)]
pub struct session_signed_pre_key {
    pub base: signal_type_base,
    pub id: u32,
    pub key_pair: *mut ec_key_pair,
    pub timestamp: u64,
    pub signature_len: usize,
    pub signature: [u8; 64],
}

// Convenience methods for signal_protocol_address
impl signal_protocol_address {
    // Builds a new address, copying the name. Objects created with
    // this method need to be explicitely dropped using drop().
    // FIXME: This only works with ascii addresses.
    pub fn new(name: &str, device_id: i32) -> Self {
        let ffi_name = CString::new(name).unwrap();
        signal_protocol_address {
            name: ffi_name.into_raw(),
            name_len: name.len(),
            device_id,
        }
    }

    // This is only safe to call when the address was created from the Rust
    // side, so we don't implement de Drop trait here.
    pub fn destroy(&self) {
        // Taking back ownership of the string.
        let _s = unsafe { CString::from_raw(self.name as *mut _) };
    }
}

impl ::std::string::ToString for signal_protocol_address {
    fn to_string(&self) -> String {
        let mut res = String::new();
        unsafe {
            let cstring = CString::from_raw(self.name as *mut _);
            if let Ok(s) = cstring.clone().into_string() {
                res = s;
                let _ptr = cstring.into_raw();
            }
        }
        res
    }
}

impl signal_protocol_sender_key_name {
    pub fn new(group_id: &str, sender_name: &str, device_id: i32) -> Self {
        let ffi_group = CString::new(group_id).unwrap();
        signal_protocol_sender_key_name {
            group_id: ffi_group.into_raw(),
            group_id_len: group_id.len(),
            sender: signal_protocol_address::new(sender_name, device_id),
        }
    }

    pub fn destroy(&self) {
        // Taking back ownership of the string.
        let _ffi_group = unsafe { CString::from_raw(self.group_id as *mut _) };
        self.sender.destroy();
    }
}

// Helpers to manage the lifetime of the key from the Rust side.
impl ec_public_key {
    pub fn addref(&mut self) {
        self.base.ref_count += 1;
    }

    pub fn unref(&mut self) {
        self.base.ref_count -= 1;
        if self.base.ref_count == 0 {
            if let Some(destroy_func) = self.base.destroy {
                unsafe {
                    destroy_func(&mut self.base);
                }
            }
        }
    }
}

impl session_pre_key {
    // Serialized the pre key as a protobuf buffer.
    #[cfg(test)]
    pub fn serialize(id: u32, public_key: &KeyArray, private_key: &KeyArray) -> Vec<u8> {
        // Create buffers from the keys.
        let pub_key_buffer = signal_buffer::from_public_key(public_key);
        let priv_key_buffer = signal_buffer::from_private_key(private_key);

        // Populate a record structure
        let record = PreKeyRecordStructure {
            id: Some(id),
            public_key: unsafe { Some((*pub_key_buffer).data_slice().to_vec()) },
            private_key: unsafe { Some((*priv_key_buffer).data_slice().to_vec()) },
        };

        // Serialize it
        let mut buffer = vec![];
        record
            .encode(&mut buffer)
            .expect("Failed to encode session pre key!");

        unsafe {
            signal_buffer_free(pub_key_buffer);
            signal_buffer_free(priv_key_buffer);
        }

        buffer
    }
}

impl session_signed_pre_key {
    #[cfg(test)]
    pub fn serialize(
        id: u32,
        public_key: &KeyArray,
        private_key: &KeyArray,
        signature: &[u8],
        timestamp: u64,
    ) -> Vec<u8> {
        // Create buffers from the keys.
        let pub_key_buffer = signal_buffer::from_public_key(public_key);
        let priv_key_buffer = signal_buffer::from_private_key(private_key);

        // Populate a record structure
        let record = SignedPreKeyRecordStructure {
            id: Some(id),
            public_key: unsafe { Some((*pub_key_buffer).data_slice().to_vec()) },
            private_key: unsafe { Some((*priv_key_buffer).data_slice().to_vec()) },
            signature: Some(signature.to_vec()),
            timestamp: Some(timestamp),
        };

        // Serialize it
        let mut buffer = vec![];
        record
            .encode(&mut buffer)
            .expect("Failed to encode session pre key!");

        unsafe {
            signal_buffer_free(pub_key_buffer);
            signal_buffer_free(priv_key_buffer);
        }

        buffer
    }
}

pub fn int_list_from_vec(from: Vec<::std::os::raw::c_int>) -> *mut signal_int_list {
    unsafe {
        let list = signal_int_list_alloc();
        for value in from {
            signal_int_list_push_back(list, value);
        }
        list
    }
}

#[derive(Debug, PartialEq)]
pub enum DecryptionError {
    InvalidMessage,
    DuplicateMessage,
    LegacyMessage,
    NoSession,
    UntrustedIdentity,
    InvalidKey,
    InvalidKeyId,
    DeserializationError,
    DecryptionCallbackFailure,
    Other,
    Unknown(c_int),
}

// From signal_protocol.h :
// #define SG_ERR_DUPLICATE_MESSAGE    -1001
// #define SG_ERR_INVALID_KEY          -1002
// #define SG_ERR_INVALID_KEY_ID       -1003
// #define SG_ERR_INVALID_MESSAGE      -1005
// #define SG_ERR_LEGACY_MESSAGE       -1007
// #define SG_ERR_NO_SESSION           -1008
// #define SG_ERR_UNTRUSTED_IDENTITY   -1010
impl ::std::convert::From<c_int> for DecryptionError {
    fn from(val: c_int) -> Self {
        match val {
            -1001 => DecryptionError::DuplicateMessage,
            -1002 => DecryptionError::InvalidKey,
            -1003 => DecryptionError::InvalidKeyId,
            -1005 => DecryptionError::InvalidMessage,
            -1007 => DecryptionError::LegacyMessage,
            -1008 => DecryptionError::NoSession,
            -1010 => DecryptionError::UntrustedIdentity,
            -2000 => DecryptionError::DeserializationError,
            -3000 => DecryptionError::Other,
            -4000 => DecryptionError::DecryptionCallbackFailure,
            other => DecryptionError::Unknown(other),
        }
    }
}

impl DecryptionError {
    pub fn as_int(&self) -> c_int {
        match *self {
            DecryptionError::DuplicateMessage => -1001,
            DecryptionError::InvalidKey => -1002,
            DecryptionError::InvalidKeyId => -1003,
            DecryptionError::InvalidMessage => -1005,
            DecryptionError::LegacyMessage => -1007,
            DecryptionError::NoSession => -1008,
            DecryptionError::UntrustedIdentity => -1010,
            DecryptionError::DeserializationError => -2000,
            DecryptionError::Other => -3000,
            DecryptionError::DecryptionCallbackFailure => -4000,
            DecryptionError::Unknown(val) => val,
        }
    }
}

#[test]
fn test_signal_address() {
    let a = signal_protocol_address::new("+123456789", 1);
    assert_eq!(a.name_len, 10);
    assert_eq!(a.device_id, 1);
    let s = a.to_string();
    assert_eq!("+123456789".to_owned(), s);
    a.destroy();
}

#[test]
fn session_pre_key_serialize() {
    let public: KeyArray = [0; 32];
    let private: KeyArray = [0; 32];

    let serialized = session_pre_key::serialize(42, &public, &private);
    assert_eq!(serialized.len(), 71);
}
