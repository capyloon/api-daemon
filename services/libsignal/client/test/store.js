// Implementation of store callbacks.

class SessionStore extends lib_libsignal.SessionStoreBase {
  constructor(service, session) {
    super(service.id, session);

    this.data = {};
  }

  display() {
    return "Session Store";
  }

  log() {
    console.log(`SessionStore: `, ...arguments);
  }

  // fn load(address: Address) -> binary
  load(address) {
    this.log(`load`, address);
    let record = this.data[`${address.name}|${address.deviceId}`];
    return Promise.resolve(record || []);
  }

  // fn get_sub_device_sessions(name: str) -> int*
  getSubDeviceSessions(name) {
    this.log(`getSubDeviceSessions`, name);
    return Promise.resolve([]);
  }

  // fn store(address: Address, record: binary)
  store(address, record) {
    this.log(`store`, address, record);
    this.data[`${address.name}|${address.deviceId}`] = record;
    return Promise.resolve();
  }

  // fn contains(address: Address) -> bool
  contains(address) {
    this.log(`contains`, address);
    return Promise.resolve(!!this.data[`${address.name}|${address.deviceId}`]);
  }

  // fn delete(address: Address) -> bool
  delete(address) {
    this.log(`delete`, address);
    delete this.data[`${address.name}|${address.deviceId}`];
    return Promise.resolve(true);
  }

  // fn delete_all_sessions(name: str) -> int
  deleteAllSessions(name) {
    this.log(`deleteAllSessions`, name);
    let count = Object.keys(this.data).length;
    this.data = {};
    return Promise.resolve(count);
  }
}

class KeyStore extends lib_libsignal.KeyStoreBase {
  constructor(service, session) {
    super(service.id, session);
  }

  display() {
    return "Key Store";
  }

  log() {
    console.log(`KeyStore: `, ...arguments);
  }

  // fn load(preKey_id: int) -> binary
  load(preKeyId) {
    this.log(`load`, preKeyId);
    return Promise.resolve([]);
  }

  // fn contains(preKey_id: int) -> bool
  contains(preKeyId) {
    this.log(`contains`, preKeyId);
    return Promise.resolve(true);
  }

  // fn remove(preKey_id: int)
  remove(preKeyId) {
    this.log(`remove`, preKeyId);
    return Promise.resolve();
  }
}

class IdentityKeyStore extends lib_libsignal.IdentityKeyStoreBase {
  constructor(service, session) {
    super(service.id, session);

    this.identity = {};
  }

  display() {
    return "Identity Key Store";
  }

  log() {
    console.log(`IdentityKeyStore: `, ...arguments);
  }

  setKeyPair(keyPair) {
    this.keyPair = keyPair;
  }

  // fn get_key_pair() -> EcKeyPair
  getKeyPair() {
    this.log(`getKeyPair`);
    if (this.keyPair) {
      return Promise.resolve(this.keyPair);
    } else {
      return Promise.reject();
    }
  }

  setLocalRegistrationId(id) {
    this.log(`setLocalRegistrationId`);
    this.registrationId = id;
  }

  // fn get_local_registration_id() -> int
  getLocalRegistrationId() {
    this.log(`getLocalRegistrationId`);
    return Promise.resolve(this.registrationId);
  }

  // fn save_identity(address: Address, key_data: binary)
  saveIdentity(address, keyData) {
    this.log(`saveIdentity`, address, keyData);
    this.identity[`${address.name}|${address.deviceId}`] = keyData;
    return Promise.resolve();
  }

  // fn is_trusted_identity(address: Address, key_data: binary) -> bool
  isTrustedIdentity(address, keyData) {
    this.log(`isTrustedIdentity`, address, keyData);
    return Promise.resolve(true);
  }
}

class SenderKeyStore extends lib_libsignal.SenderKeyStoreBase {
  constructor(service, session) {
    super(service.id, session);
  }

  display() {
    return "Sender Key Store";
  }

  log() {
    console.log(`SenderKeyStore `, ...arguments);
  }

  // fn store(sender_key_name: SenderKeyName, record: binary) -> bool
  store(senderKeyName, record) {
    this.log(`store`, senderKeyName);
    this.record = record;
    return Promise.resolve(true);
  }

  // fn load(sender_key_name: SenderKeyName) -> binary
  load(senderKeyName) {
    this.log(`load`, senderKeyName);
    return Promise.resolve(this.record || []);
  }
}

function createStoreContextFor(tester) {
  return {
    sessionStore: new SessionStore(tester.service, tester.session),
    preKeyStore: new KeyStore(tester.service, tester.session),
    signedPreKeyStore: new KeyStore(tester.service, tester.session),
    identityKeyStore: new IdentityKeyStore(tester.service, tester.session),
    senderKeyStore: new SenderKeyStore(tester.service, tester.session),
  };
}

class DecryptionCallbackWrapper extends lib_libsignal.DecryptionCallbackBase {
  constructor(tester, closure) {
    super(tester.service, tester.session);
    this.closure = closure;
  }

  display() {
    return "Decryption Callback";
  }

  callback(plaintext) {
    return this.closure(plaintext);
  }
}

// class TextSecure {
//   // Serializes a signed pre key in the TextSecure protobuf format.
//   serializeSignedPrekey(preKey) {
//     // The public key needs an extra byte in front when serialized...
//     let pubKeyBuffer = new Uint8Array(33);
//     pubKeyBuffer[0] = 0x05;
//     pubKeyBuffer.set(preKey.keyPair.publicKey, 1);

//     let record = textsecure.SignedPreKeyRecordStructure.create({
//       id: preKey.id,
//       publicKey: pubKeyBuffer,
//       privateKey: preKey.keyPair.privateKey,
//       signature: preKey.signature,
//       timestamp: preKey.timestamp,
//     });

//     return textsecure.SignedPreKeyRecordStructure.encode(record).finish();
//   }

//   // Serializes a pre key in the TextSecure protobuf format.
//   serializePreKey(preKey) {
//     // The public key needs an extra byte in front when serialized...
//     let pubKeyBuffer = new Uint8Array(33);
//     pubKeyBuffer[0] = 0x05;
//     pubKeyBuffer.set(preKey.keyPair.publicKey, 1);

//     let record = textsecure.PreKeyRecordStructure.create({
//       id: preKey.id,
//       publicKey: pubKeyBuffer,
//       privateKey: preKey.keyPair.privateKey,
//     });

//     return textsecure.PreKeyRecordStructure.encode(record).finish();
//   }
// }
