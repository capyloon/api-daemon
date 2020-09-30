use crate::generated::ffi::*;
use std::ptr::null_mut;
use crate::store_context::StoreContext;
use crate::signal_context::{SignalContext, SignalContextPtr};
use std::os::raw::c_int;

// Wrapper around a group session_builder
pub type GroupSessionBuilderPtr = *mut group_session_builder;

pub struct GroupSessionBuilder {
    native: GroupSessionBuilderPtr,
    signal_ctxt: SignalContextPtr,
}

unsafe impl Sync for GroupSessionBuilder {}
unsafe impl Send for GroupSessionBuilder {}

impl GroupSessionBuilder {
    pub fn new(store: &StoreContext, ctxt: &SignalContext) -> Option<Self> {
        let mut builder: GroupSessionBuilderPtr = null_mut();
        if unsafe { group_session_builder_create(&mut builder, store.native(), ctxt.native()) } == 0
        {
            return Some(GroupSessionBuilder {
                native: builder,
                signal_ctxt: ctxt.native(),
            });
        } else {
            debug!("Error in group_session_builder_create");
        }

        None
    }

    pub fn native(&self) -> GroupSessionBuilderPtr {
        self.native
    }

    pub fn global_context(&self) -> SignalContextPtr {
        self.signal_ctxt
    }

    // Construct a group session for sending messages.
    // The error is the negative return code from the C library.
    pub fn create_session(
        &self,
        group_id: &str,
        sender_name: &str,
        device_id: i32,
    ) -> Result<SenderKeyDistributionMessage, c_int> {
        // Build a signal_protocol_sender_key_name from the parameters. Its lifetime
        // will be managed by the Rust side.
        let sender_key_name =
            signal_protocol_sender_key_name::new(group_id, sender_name, device_id);

        type ResPtr = *mut sender_key_distribution_message;
        let mut res: ResPtr = null_mut();

        let ret = unsafe {
            group_session_builder_create_session(self.native, &mut res, &sender_key_name)
        };

        sender_key_name.destroy();

        if ret < 0 {
            Err(ret)
        } else {
            Ok(SenderKeyDistributionMessage::from(res))
        }
    }

    // Construct a group session for receiving messages from the sender.
    // The error is the negative return code from the C library.
    pub fn process_session(
        &self,
        group_id: &str,
        sender_name: &str,
        device_id: i32,
        distribution_message: &SenderKeyDistributionMessage,
    ) -> Result<(), c_int> {
        // Build a signal_protocol_sender_key_name from the parameters. Its lifetime
        // will be managed by the Rust side.
        let sender_key_name =
            signal_protocol_sender_key_name::new(group_id, sender_name, device_id);

        let ret = unsafe {
            group_session_builder_process_session(
                self.native,
                &sender_key_name,
                distribution_message.native(),
            )
        };

        sender_key_name.destroy();

        if ret < 0 {
            Err(ret)
        } else {
            Ok(())
        }
    }
}

impl Drop for GroupSessionBuilder {
    fn drop(&mut self) {
        debug!("Dropping GroupSessionBuilder");
        unsafe {
            group_session_builder_free(self.native);
        }
    }
}

// A Rust wrapper around sender_key_distribution_message
pub struct SenderKeyDistributionMessage {
    native: *mut sender_key_distribution_message,
}

impl SenderKeyDistributionMessage {
    pub fn from(from: *mut sender_key_distribution_message) -> SenderKeyDistributionMessage {
        SenderKeyDistributionMessage { native: from }
    }

    pub fn native(&self) -> *mut sender_key_distribution_message {
        self.native
    }

    /// Serializes the message into a format suitable for exchange.
    pub fn serialize(&self) -> Vec<u8> {
        unsafe {
            // ciphertext_message_get_serialized() doesn't allocate memory, so there is no
            // memory management to deal with.
            let buffer =
                ciphertext_message_get_serialized(self.native as *const ciphertext_message);

            (*buffer).data_slice().to_vec().clone()
        }
    }

    /// Creates a message from a serialized buffer.
    pub fn deserialize(buffer: &[u8], global_context: SignalContextPtr) -> Option<Self> {
        unsafe {
            type NativePtr = *mut sender_key_distribution_message;
            let mut native: NativePtr = null_mut();
            let res = sender_key_distribution_message_deserialize(
                &mut native,
                buffer.as_ptr(),
                buffer.len(),
                global_context,
            );

            if res == 0 {
                return Some(SenderKeyDistributionMessage { native });
            }
        }
        None
    }
}

impl Drop for SenderKeyDistributionMessage {
    fn drop(&mut self) {
        unsafe {
            sender_key_distribution_message_destroy(&mut (*self.native).base_message.base);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::group_cipher::GroupCipher;
    use crate::signal_context::SignalContext;
    use crate::store_context::StoreContext;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::os::raw::c_void;
    use std::os::raw::c_int;

    extern "C" fn decrypt_callback(
        _cipher: *mut group_cipher,
        _plaintext: *mut signal_buffer,
        _decrypt_context: *mut c_void,
    ) -> c_int {
        0
    }

    // Setup a test sender key store.
    struct TestStoreData {
        record: Cell<Vec<u8>>,
        name: String,
    }

    impl TestStoreData {
        pub fn new(name: &str) -> Self {
            println!("Creating TestStoreData {}", name);
            TestStoreData {
                name: name.to_owned(),
                record: Cell::new(vec![]),
            }
        }
    }

    impl Drop for TestStoreData {
        fn drop(&mut self) {
            println!("Dropping TestStoreData {}", self.name);
        }
    }

    // @return 0 on success, negative on failure
    extern "C" fn store_sender_key(
        _sender_key_name: *const signal_protocol_sender_key_name,
        record: *mut u8,
        record_len: usize,
        _user_record: *mut u8,
        _user_record_len: usize,
        user_data: *mut c_void,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        println!("store_sender_key {}", data.name);

        unsafe {
            let vec = Vec::from_raw_parts(record, record_len, record_len);
            data.record.set(vec.clone());
            ::std::mem::forget(vec);
        }
        let _ = Rc::into_raw(data);
        0
    }

    // @return 1 if the record was loaded, 0 if the record was not found, negative on failure
    extern "C" fn load_sender_key(
        record: *mut *mut signal_buffer,
        _user_record: *mut *mut signal_buffer,
        _sender_key_name: *const signal_protocol_sender_key_name,
        user_data: *mut c_void,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        println!("load_sender_key {}", data.name);

        let res = unsafe {
            let vec = &*data.record.as_ptr();
            if vec.len() == 0 {
                0
            } else {
                // println!("vec len={}", vec.len());
                *record = signal_buffer::from_slice(vec);
                1
            }
        };

        let _ = Rc::into_raw(data);
        res
    }

    fn get_sender_key_store(user_data: *mut c_void) -> signal_protocol_sender_key_store {
        signal_protocol_sender_key_store {
            store_sender_key: Some(store_sender_key),
            load_sender_key: Some(load_sender_key),
            destroy_func: None,
            user_data,
        }
    }

    #[test]
    fn create_group_session_builder() {
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let session_builder = GroupSessionBuilder::new(&s_context, &g_context);
        assert!(session_builder.is_some());
    }

    #[test]
    fn new_group_session() {
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let bob_store_data = Rc::new(TestStoreData::new("Bob"));
        let bob_user_data = Rc::into_raw(bob_store_data) as *mut c_void;
        s_context.set_sender_key_store(get_sender_key_store(bob_user_data));
        let session_builder = GroupSessionBuilder::new(&s_context, &g_context).unwrap();

        let message = session_builder
            .create_session("group1", "sender1", 42)
            .unwrap();

        let res = session_builder.process_session("group1", "sender1", 42, &message);
        assert!(res.is_ok());

        // Make sure we don't leak.
        unsafe {
            let _ : Rc<TestStoreData> = Rc::from_raw(bob_user_data as *const _);
        }
    }

    #[test]
    fn serialize_sender_key_distribution_message() {
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let bob_store_data = Rc::new(TestStoreData::new("Bob"));
        let bob_user_data = Rc::into_raw(bob_store_data) as *mut c_void;
        s_context.set_sender_key_store(get_sender_key_store(bob_user_data));
        let session_builder = GroupSessionBuilder::new(&s_context, &g_context).unwrap();

        let sent_message = session_builder
            .create_session("group1", "sender1", 42)
            .unwrap();

        let serialized = sent_message.serialize();
        assert_ne!(serialized.len(), 0);

        let _received_message =
            SenderKeyDistributionMessage::deserialize(&serialized, g_context.native()).unwrap();

        // Make sure we don't leak.
        unsafe {
            let _ : Rc<TestStoreData> = Rc::from_raw(bob_user_data as *const _);
        }
    }

    #[test]
    fn group_encrypt_decrypt() {
        // Port of test_group_cipher.c test_basic_encrypt_decrypt

        let g_context = SignalContext::new().unwrap();

        // Create the test stores.
        let alice_store = StoreContext::new(&g_context).unwrap();
        let alice_store_data = Rc::new(TestStoreData::new("Alice"));
        let alice_user_data = Rc::into_raw(alice_store_data) as *mut c_void;
        alice_store.set_sender_key_store(get_sender_key_store(alice_user_data));

        let bob_store = StoreContext::new(&g_context).unwrap();
        let bob_store_data = Rc::new(TestStoreData::new("Bob"));
        let bob_user_data = Rc::into_raw(bob_store_data) as *mut c_void;
        bob_store.set_sender_key_store(get_sender_key_store(bob_user_data));

        // Create the session builders.
        let alice_session_builder = GroupSessionBuilder::new(&alice_store, &g_context).unwrap();
        let bob_session_builder = GroupSessionBuilder::new(&bob_store, &g_context).unwrap();

        // Create the group ciphers.
        let alice_cipher =
            GroupCipher::new(&alice_store, &g_context, "group 1", "sender 1", 42).unwrap();
        alice_cipher.set_callback(decrypt_callback, null_mut());
        let bob_cipher =
            GroupCipher::new(&bob_store, &g_context, "group 1", "sender 1", 42).unwrap();
        bob_cipher.set_callback(decrypt_callback, null_mut());

        // Create the sender key distribution messages.
        let sent_alice_distribution_message = alice_session_builder
            .create_session("group 1", "sender 1", 42)
            .unwrap();

        let serialized = sent_alice_distribution_message.serialize();

        let received_alice_distribution_message =
            SenderKeyDistributionMessage::deserialize(&serialized, g_context.native()).unwrap();

        // Processing Alice's distribution message.
        bob_session_builder
            .process_session(
                "group 1",
                "sender 1",
                42,
                &received_alice_distribution_message,
            )
            .unwrap();

        // Encrypt a message from Alice.
        let message = b"signal group message";
        let ciphertext_from_alice = alice_cipher.encrypt(message).unwrap();

        // Have Bob decrypt the message.
        let plaintext = bob_cipher.decrypt(&ciphertext_from_alice).unwrap();

        // Make sure we don't leak.
        unsafe {
            let _ : Rc<TestStoreData> = Rc::from_raw(alice_user_data as *const _);
            let _ : Rc<TestStoreData> = Rc::from_raw(bob_user_data as *const _);
        }

        assert_eq!(plaintext, message);
    }
}
