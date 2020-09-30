use crate::generated::ffi::*;
use std::ptr::null_mut;
use crate::store_context::StoreContext;
use crate::signal_context::SignalContext;
use std::os::raw::c_int;

// Wrapper around a session_builder
pub type SessionBuilderPtr = *mut session_builder;

pub struct SessionBuilder {
    native: SessionBuilderPtr,
}

unsafe impl Sync for SessionBuilder {}
unsafe impl Send for SessionBuilder {}

impl SessionBuilder {
    pub fn new(
        store: &StoreContext,
        remote_address: *const signal_protocol_address,
        ctxt: &SignalContext,
    ) -> Option<Self> {
        let mut builder: SessionBuilderPtr = null_mut();
        if unsafe {
            session_builder_create(&mut builder, store.native(), remote_address, ctxt.native())
        } == 0
        {
            return Some(SessionBuilder { native: builder });
        } else {
            debug!("Error in session_builder_create");
        }

        None
    }

    pub fn native(&self) -> SessionBuilderPtr {
        self.native
    }

    pub fn process_pre_key_bundle(&self, bundle: &SessionPreKeyBundle) -> bool {
        let res = unsafe { session_builder_process_pre_key_bundle(self.native, bundle.native()) };
        if res != 0 {
            debug!("process_pre_key_bundle failed with error code {}", res);
            #[cfg(test)]
            {
                println!("process_pre_key_bundle failed with error code {}", res);
            }
        }
        res == 0
    }
}

impl Drop for SessionBuilder {
    fn drop(&mut self) {
        debug!("Dropping SessionBuilder");
        unsafe {
            session_builder_free(self.native);
        }
    }
}

// Wrapper around session_pre_key_bundle
pub type SessionPreKeyBundlePtr = *mut session_pre_key_bundle;

pub struct SessionPreKeyBundle {
    native: SessionPreKeyBundlePtr,
}

fn create_ec_public_key(ctxt: &SignalContext, data: &KeyArray) -> *mut ec_public_key {
    type KeyPairPtr = *mut ec_key_pair;
    unsafe {
        let mut key_pair: KeyPairPtr = null_mut(); //ec_key_pair *key_pair;
        let _result = curve_generate_key_pair(ctxt.native(), &mut key_pair);

        let public_key = ec_key_pair_get_public(key_pair);
        (*public_key).addref();

        assert_eq!((*public_key).data.len(), data.len());
        (*public_key).data[..data.len()].clone_from_slice(&data[..]);

        ec_key_pair_destroy(&mut (*key_pair).base);

        public_key
    }
}

impl SessionPreKeyBundle {
    pub fn new(
        registration_id: u32,
        device_id: u32,
        pre_key_id: u32,
        pre_key_public: Option<&KeyArray>,
        signed_pre_key_id: u32,
        signed_pre_key_public: &KeyArray,
        signed_pre_key_signature: &[u8],
        identity_key: &KeyArray,
    ) -> Option<Self> {
        // session_pre_key_bundle_create(bundle: *mut *mut session_pre_key_bundle,
        //                               registration_id: u32,
        //                               device_id: c_int,
        //                               pre_key_id: u32,
        //                               pre_key_public: *mut ec_public_key,
        //                               signed_pre_key_id: u32,
        //                               signed_pre_key_public: *mut ec_public_key,
        //                               signed_pre_key_signature_data: *const u8,
        //                               signed_pre_key_signature_len: usize,
        //                               identity_key: *mut ec_public_key)

        let ctxt = SignalContext::new().expect("Failed to create SignalContext !");
        let mut ptr: SessionPreKeyBundlePtr = null_mut();
        if unsafe {
            let p_k_p = match pre_key_public {
                Some(key) => create_ec_public_key(&ctxt, key),
                None => ::std::ptr::null_mut(),
            };

            let sp_k_p = create_ec_public_key(&ctxt, signed_pre_key_public);
            let id_key = create_ec_public_key(&ctxt, identity_key);

            let res = session_pre_key_bundle_create(
                &mut ptr,
                registration_id,
                device_id as c_int,
                pre_key_id,
                p_k_p,
                signed_pre_key_id,
                sp_k_p,
                signed_pre_key_signature.as_ptr(),
                signed_pre_key_signature.len(),
                id_key,
            );

            if !p_k_p.is_null() {
                (*p_k_p).unref();
            }
            (*sp_k_p).unref();
            (*id_key).unref();

            res
        } == 0
        {
            return Some(SessionPreKeyBundle { native: ptr });
        }

        None
    }

    pub fn native(&self) -> SessionPreKeyBundlePtr {
        self.native
    }
}

impl Drop for SessionPreKeyBundle {
    fn drop(&mut self) {
        debug!("Dropping SessionPreKeyBundle");
        unsafe {
            session_pre_key_bundle_destroy(&mut (*self.native).base);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ffi::DecryptionError;
    use crate::session_cipher::SessionCipher;
    use crate::signal_context::SignalContext;
    use crate::store_context::StoreContext;
    use std::rc::Rc;
    use std::os::raw::c_void;
    use std::cell::Cell;
    use super::*;

    extern "C" fn decrypt_callback(
        _cipher: *mut session_cipher,
        _plaintext: *mut signal_buffer,
        _decrypt_context: *mut c_void,
    ) -> c_int {
        0
    }

    extern "C" fn decrypt_callback_failure(
        _cipher: *mut session_cipher,
        _plaintext: *mut signal_buffer,
        _decrypt_context: *mut c_void,
    ) -> c_int {
        DecryptionError::Other.as_int()
    }

    struct TestStoreData {
        public_key: KeyArray,
        private_key: KeyArray,
        registration_id: u32,
        pre_key: Cell<Vec<u8>>,
        session: Cell<Vec<u8>>,
        signed_pre_key: Cell<Vec<u8>>,
        name: String,
    }

    fn create_test_store_data(ctxt: &SignalContext, name: &str) -> TestStoreData {
        let (id_public, id_private) = ctxt.generate_identity_key_pair().unwrap();

        return TestStoreData {
            public_key: id_public,
            private_key: id_private,
            registration_id: ctxt.get_registration_id(false).unwrap(),
            pre_key: Cell::new(vec![]),
            session: Cell::new(vec![]),
            signed_pre_key: Cell::new(vec![]),
            name: name.to_owned(),
        };
    }

    // Setup for a test identity store.
    extern "C" fn is_trusted_identity(
        address: *const signal_protocol_address,
        _key_data: *mut u8,
        _key_len: usize,
        _user_data: *mut c_void,
    ) -> c_int {
        println!("is_trusted_identity {}", unsafe { (*address).to_string() });
        1
    }

    extern "C" fn get_identity_key_pair(
        public_data: *mut SignalBufferPtr,
        private_data: *mut SignalBufferPtr,
        user_data: *mut c_void,
    ) -> c_int {
        println!("get_identity_key_pair {:?} {:?}", public_data, private_data);
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        unsafe {
            *public_data = signal_buffer::from_public_key(&data.public_key);
            *private_data = signal_buffer::from_slice(&data.private_key);
        }
        let _ = Rc::into_raw(data);
        0
    }

    extern "C" fn get_local_registration_id(
        user_data: *mut c_void,
        registration_id: *mut u32,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        unsafe {
            *registration_id = data.registration_id;
        }
        let _ = Rc::into_raw(data);
        0
    }

    extern "C" fn save_identity(
        _address: *const signal_protocol_address,
        _key_data: *mut u8,
        _key_len: usize,
        _user_data: *mut c_void,
    ) -> c_int {
        println!("save_identity");
        // We don't properly save it since we don't manage several identities
        // in is_trusted_identity.
        0
    }

    fn get_identity_store(user_data: *mut c_void) -> signal_protocol_identity_key_store {
        signal_protocol_identity_key_store {
            get_identity_key_pair: Some(get_identity_key_pair),
            get_local_registration_id: Some(get_local_registration_id),
            save_identity: Some(save_identity),
            is_trusted_identity: Some(is_trusted_identity),
            destroy_func: None,
            user_data,
        }
    }

    // Set up for a test session store.
    extern "C" fn load_session_func(
        record: *mut *mut signal_buffer,
        _user_record: *mut *mut signal_buffer,
        _address: *const signal_protocol_address,
        user_data: *mut c_void,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        println!("load_session_func for {}", data.name);
        let res = unsafe {
            let vec = &*data.session.as_ptr();
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

    extern "C" fn store_session_func(
        _address: *const signal_protocol_address,
        record: *mut u8,
        record_len: usize,
        _user_record: *mut u8,
        _user_record_len: usize,
        user_data: *mut c_void,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        println!(
            "store_session_func for {}, len is {}",
            data.name, record_len
        );
        unsafe {
            let vec = Vec::from_raw_parts(record, record_len, record_len);
            data.session.set(vec.clone());
            ::std::mem::forget(vec);
        }
        let _ = Rc::into_raw(data);
        0
    }

    fn get_session_store(user_data: *mut c_void) -> signal_protocol_session_store {
        signal_protocol_session_store {
            load_session_func: Some(load_session_func),
            get_sub_device_sessions_func: None,
            store_session_func: Some(store_session_func),
            contains_session_func: None,
            delete_session_func: None,
            delete_all_sessions_func: None,
            destroy_func: None,
            user_data,
        }
    }

    // Set up for a test signed pre key store.
    extern "C" fn load_signed_pre_key(
        record: *mut SignalBufferPtr,
        signed_pre_key_id: u32,
        user_data: *mut c_void,
    ) -> c_int {
        println!("load_signed_pre_key for id {}", signed_pre_key_id);
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        let res = unsafe {
            let vec = &*data.signed_pre_key.as_ptr();
            if vec.len() == 0 {
                -1003 // SG_ERR_INVALID_KEY_ID
            } else {
                // println!("vec len={}", vec.len());
                *record = signal_buffer::from_slice(vec);
                0
            }
        };
        let _ = Rc::into_raw(data);
        res
    }

    extern "C" fn contains_signed_pre_key(
        signed_pre_key_id: u32,
        _user_data: *mut c_void,
    ) -> c_int {
        println!("contains_signed_pre_key for id {}", signed_pre_key_id);
        0
    }

    extern "C" fn remove_signed_pre_key(signed_pre_key_id: u32, _user_data: *mut c_void) -> c_int {
        println!("remove_signed_pre_key for id {}", signed_pre_key_id);
        0
    }

    fn get_signed_pre_key_store(user_data: *mut c_void) -> signal_protocol_signed_pre_key_store {
        signal_protocol_signed_pre_key_store {
            load_signed_pre_key: Some(load_signed_pre_key),
            store_signed_pre_key: None,
            contains_signed_pre_key: Some(contains_signed_pre_key),
            remove_signed_pre_key: Some(remove_signed_pre_key),
            destroy_func: None,
            user_data,
        }
    }

    // Set up a test pre key store.
    extern "C" fn load_pre_key(
        record: *mut SignalBufferPtr,
        pre_key_id: u32,
        user_data: *mut c_void,
    ) -> c_int {
        let data: Rc<TestStoreData> = unsafe { Rc::from_raw(user_data as *const TestStoreData) };
        println!("load_pre_key for {} id {}", data.name, pre_key_id);
        let res = unsafe {
            let vec = &*data.pre_key.as_ptr();
            // panic!("load_pre_key len={}", vec.len());
            if vec.len() == 0 {
                -1003 // SG_ERR_INVALID_KEY_ID
            } else {
                // println!("vec len={}", vec.len());
                *record = signal_buffer::from_slice(vec);
                0
            }
        };
        let _ = Rc::into_raw(data);
        res
    }

    extern "C" fn contains_pre_key(pre_key_id: u32, _user_data: *mut c_void) -> c_int {
        println!("contains_pre_key for id {}", pre_key_id);
        1
    }

    extern "C" fn remove_pre_key(pre_key_id: u32, _user_data: *mut c_void) -> c_int {
        println!("remove_pre_key for id {}", pre_key_id);
        0
    }

    fn get_pre_key_store(user_data: *mut c_void) -> signal_protocol_pre_key_store {
        signal_protocol_pre_key_store {
            load_pre_key: Some(load_pre_key),
            store_pre_key: None,
            contains_pre_key: Some(contains_pre_key),
            remove_pre_key: Some(remove_pre_key),
            destroy_func: None,
            user_data,
        }
    }

    #[test]
    fn create_session_builder() {
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let address = signal_protocol_address::new("+1234567890", 42);
        let session_builder = SessionBuilder::new(&s_context, &address, &g_context);
        assert!(session_builder.is_some());
        address.destroy();
    }

    #[test]
    fn session_bundle() {
        // Create all contexts and builder.
        let g_context = SignalContext::new().unwrap();
        let s_context = StoreContext::new(&g_context).unwrap();
        let store_data = Rc::new(create_test_store_data(&g_context, "user1"));
        let user_data = Rc::into_raw(store_data) as *mut c_void;
        s_context.set_identity_key_store(get_identity_store(user_data));
        s_context.set_session_store(get_session_store(user_data));

        let address = signal_protocol_address::new("+1234567890", 42);
        let session_builder = SessionBuilder::new(&s_context, &address, &g_context).unwrap();

        // Get registration id and keys.
        let registration_id = g_context.get_registration_id(false).unwrap();
        let pre_key = g_context.generate_pre_keys(1, 1).unwrap()[0];

        let (id_public, id_private) = g_context.generate_identity_key_pair().unwrap();
        // signed_pre_key = (id, public_key, private_key, timestamp, signature)
        let signed_pre_key = g_context
            .generate_signed_pre_key(&id_public, &id_private, registration_id, 0)
            .unwrap();

        let bundle = SessionPreKeyBundle::new(
            registration_id,
            1,
            pre_key.0,
            Some(&pre_key.1),
            signed_pre_key.0,
            &signed_pre_key.1,
            &signed_pre_key.4,
            &id_public,
        ).unwrap();

        assert!(session_builder.process_pre_key_bundle(&bundle));

        // Now create a session cipher and use it to encrypt and decrypt.
        let session_cipher = SessionCipher::new(&s_context, &address, &g_context).unwrap();
        session_cipher.set_callback(decrypt_callback, null_mut());

        let res = session_cipher
            .encrypt("Hello World! Please encrypt this message!".as_bytes())
            .unwrap();
        assert_eq!(res.0, 3); // message type.
        assert_eq!(res.1.len(), 179);

        let plaintext = session_cipher.decrypt_pre_key_message(&res.1);
        assert_eq!(plaintext.err().unwrap(), DecryptionError::InvalidMessage);

        let plaintext = session_cipher.decrypt_message(&res.1);
        assert_eq!(
            plaintext.err().unwrap(),
            DecryptionError::DeserializationError
        );

        address.destroy();
        unsafe {
            let _ : Rc<TestStoreData> = Rc::from_raw(user_data as *const _);
        }
    }

    #[test]
    fn one_to_one_session() {
        // This is testing that Alice & Bob can actually exchange messages.
        let bob_jid = "Bob's JID";
        let alice_jid = "Alice's JID";

        // Each user gets a new context with its own registration id.
        let bob_context = SignalContext::new().unwrap();
        let alice_context = SignalContext::new().unwrap();

        // Set up Bob's store.
        let bob_store = StoreContext::new(&bob_context).unwrap();
        let bob_store_data = Rc::new(create_test_store_data(&bob_context, "Bob"));
        let bob_user_data = Rc::into_raw(bob_store_data) as *mut c_void;
        bob_store.set_identity_key_store(get_identity_store(bob_user_data));
        bob_store.set_session_store(get_session_store(bob_user_data));
        bob_store.set_pre_key_store(get_pre_key_store(bob_user_data));
        bob_store.set_signed_pre_key_store(get_signed_pre_key_store(bob_user_data));

        // Set up Alice's store.
        let alice_store = StoreContext::new(&alice_context).unwrap();
        let alice_store_data = Rc::new(create_test_store_data(&alice_context, "Alice"));
        let alice_user_data = Rc::into_raw(alice_store_data) as *mut c_void;
        alice_store.set_identity_key_store(get_identity_store(alice_user_data));
        alice_store.set_session_store(get_session_store(alice_user_data));
        alice_store.set_pre_key_store(get_pre_key_store(alice_user_data));
        alice_store.set_signed_pre_key_store(get_signed_pre_key_store(alice_user_data));

        // Prevents leaking TestStoreData buffers.
        let bob_user_data = unsafe { Rc::from_raw(bob_user_data as *const TestStoreData) };
        let _alice_user_data = unsafe { Rc::from_raw(alice_user_data as *const TestStoreData) };

        let alice_address = signal_protocol_address::new(alice_jid, 42);
        let bob_address = signal_protocol_address::new(bob_jid, 24);

        let alice_session_builder =
            SessionBuilder::new(&alice_store, &alice_address, &alice_context).unwrap();
        let bob_registration_id = bob_user_data.registration_id;
        // let bob_ident_keypair = (bob_user_data.public_key, bob_user_data.private_key);

        let pre_key = bob_context.generate_pre_keys(1, 1).unwrap()[0];

        bob_user_data.pre_key.set(session_pre_key::serialize(
            pre_key.0,
            &pre_key.1,
            &pre_key.2,
        ));

        // return (id, public_key, private_key, timestamp, signature)
        let signed_pre_key = bob_context
            .generate_signed_pre_key(
                &bob_user_data.public_key,
                &bob_user_data.private_key,
                bob_registration_id,
                0,
            )
            .unwrap();

        bob_user_data
            .signed_pre_key
            .set(session_signed_pre_key::serialize(
                signed_pre_key.0,
                &signed_pre_key.1,
                &signed_pre_key.2,
                &signed_pre_key.4,
                signed_pre_key.3,
            ));

        // Create a bundle with a pre key
        let bundle = SessionPreKeyBundle::new(
            bob_registration_id,
            1,
            pre_key.0,
            Some(&pre_key.1),
            signed_pre_key.0,
            &signed_pre_key.1,
            &signed_pre_key.4,
            &bob_user_data.public_key,
        ).unwrap();

        assert!(alice_session_builder.process_pre_key_bundle(&bundle));

        let alice_cipher = SessionCipher::new(&alice_store, &bob_address, &alice_context).unwrap();
        alice_cipher.set_callback(decrypt_callback, null_mut());
        let cipher_message = alice_cipher
            .encrypt("Hello Bob! I am Alice!".as_bytes())
            .unwrap();
        assert_eq!(cipher_message.0, 3); // message type.
        assert_eq!(cipher_message.1.len(), 163);

        let bob_cipher = SessionCipher::new(&bob_store, &alice_address, &bob_context).unwrap();
        bob_cipher.set_callback(decrypt_callback_failure, null_mut());
        let plaintext = bob_cipher.decrypt_pre_key_message(&cipher_message.1);
        assert_eq!(plaintext.err(), Some(DecryptionError::Other));

        bob_cipher.set_callback(decrypt_callback, null_mut());
        let plaintext = bob_cipher.decrypt_pre_key_message(&cipher_message.1);
        assert!(plaintext.is_ok());
        assert_eq!(plaintext.unwrap(), "Hello Bob! I am Alice!".as_bytes());

        // Create a bundle without a pre key
        let bundle = SessionPreKeyBundle::new(
            bob_registration_id,
            1,
            pre_key.0,
            None,
            signed_pre_key.0,
            &signed_pre_key.1,
            &signed_pre_key.4,
            &bob_user_data.public_key,
        ).unwrap();

        assert!(alice_session_builder.process_pre_key_bundle(&bundle));

        let alice_cipher = SessionCipher::new(&alice_store, &bob_address, &alice_context).unwrap();
        alice_cipher.set_callback(decrypt_callback, null_mut());
        let cipher_message = alice_cipher
            .encrypt("Hello Bob! I am Alice!".as_bytes())
            .unwrap();
        assert_eq!(cipher_message.0, 3); // message type.
        assert_eq!(cipher_message.1.len(), 161);

        let bob_cipher = SessionCipher::new(&bob_store, &alice_address, &bob_context).unwrap();
        bob_cipher.set_callback(decrypt_callback_failure, null_mut());
        let plaintext = bob_cipher.decrypt_pre_key_message(&cipher_message.1);
        assert_eq!(plaintext.err(), Some(DecryptionError::Other));

        bob_cipher.set_callback(decrypt_callback, null_mut());
        let plaintext = bob_cipher.decrypt_pre_key_message(&cipher_message.1);
        assert!(plaintext.is_ok());
        assert_eq!(plaintext.unwrap(), "Hello Bob! I am Alice!".as_bytes());

        let reg_id = alice_cipher.remote_registration_id();
        assert!(reg_id.is_some());
        assert_eq!(reg_id.unwrap(), bob_user_data.registration_id);

        alice_address.destroy();
        bob_address.destroy();
    }
}
