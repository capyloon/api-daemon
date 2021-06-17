// Implementation of store callbacks.

class SessionStore extends lib_libsignal.SessionStoreBase {
  constructor(service, session, name) {
    super(service.id, session);

    this.data = {};
    this.name = name;
  }

  display() {
    return "Session Store";
  }

  log() {
    console.log(`${this.name} SessionStore: `, ...arguments);
  }

  // fn load(address: Address) -> binary
  load(address) {
    this.log(`load`, address);
    let record = this.data[`${address.name}|${address.deviceId}`];
    return Promise.resolve(record || new Uint8Array([]));
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
  constructor(service, session, name) {
    super(service.id, session);
    this.record = {};
    this.name = name;
  }

  display() {
    return "Key Store";
  }

  log() {
    console.log(`${this.name} KeyStore: `, ...arguments);
  }

  store(preKeyId, record) {
    this.log(`store`, preKeyId);
    this.record[preKeyId] = record;
  }

  // fn load(preKey_id: int) -> binary
  load(preKeyId) {
    this.log(`load`, preKeyId, this.record[preKeyId]);
    return Promise.resolve(this.record[preKeyId] || new Uint8Array([]));
  }

  // fn contains(preKey_id: int) -> bool
  contains(preKeyId) {
    this.log(`contains`, preKeyId);
    return Promise.resolve(!!this.record[preKeyId]);
  }

  // fn remove(preKey_id: int)
  remove(preKeyId) {
    this.log(`remove`, preKeyId);
    delete this.record[preKeyId];
    return Promise.resolve();
  }
}

class IdentityKeyStore extends lib_libsignal.IdentityKeyStoreBase {
  constructor(service, session, name = "User") {
    super(service.id, session);

    this.identity = {};
    this.registrationId = null;
    this.name = name;
  }

  display() {
    return "Identity Key Store";
  }

  log() {
    console.log(`${this.name} IdentityKeyStore: `, ...arguments);
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
    this.log(`setLocalRegistrationId ${id}`);
    this.registrationId = id;
  }

  // fn get_local_registration_id() -> int
  getLocalRegistrationId() {
    this.log(`getLocalRegistrationId -> ${this.registrationId}`);
    if (this.registration_id !== null) {
      return Promise.resolve(this.registration_id);
    } else {
      return Promise.reject();
    }
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
    return Promise.resolve(this.record || new Uint8Array([]));
  }
}

function createStoreContextFor(tester, name) {
  return {
    sessionStore: new SessionStore(tester.service, tester.session, name),
    preKeyStore: new KeyStore(tester.service, tester.session, name),
    signedPreKeyStore: new KeyStore(tester.service, tester.session, name),
    identityKeyStore: new IdentityKeyStore(
      tester.service,
      tester.session,
      name
    ),
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

function cryptoRandomUint32() {
  const randomKeyArray = new Uint32Array(1);
  window.crypto.getRandomValues(randomKeyArray);
  return randomKeyArray[0];
}

async function createUser(tester, address) {
  let context;
  let deviceId = address.deviceId;
  await tester.assert_eq(
    `getContext for user ${address.name} (${address.deviceId})`,
    (service) => service.newGlobalContext(),
    true,
    (result) => {
      context = result;
      return !!result;
    }
  );

  let registrationId = await context.generateRegistrationId();
  let preKeys = await context.generatePreKeys(1, 100);
  let preKey = preKeys[0];
  let identityKey = await context.generateIdentityKeyPair();
  let signedPreKey = await context.generateSignedPreKey(
    identityKey,
    cryptoRandomUint32(),
    Date.now()
  );

  let preKeyBundle = {
    registrationId,
    deviceId,
    preKeyId: preKey.id,
    preKeyPublic: preKey.keyPair.publicKey,
    signedPreKeyId: signedPreKey.id,
    signedPreKeyPublic: signedPreKey.keyPair.publicKey,
    signedPreKeySignature: signedPreKey.signature,
    identityKey: identityKey.publicKey,
  };

  // Create a store context.
  let storeContext = createStoreContextFor(tester, address.name);
  storeContext.identityKeyStore.setKeyPair(identityKey);
  storeContext.identityKeyStore.setLocalRegistrationId(registrationId);

  return {
    context,
    registrationId,
    preKeys,
    identityKey,
    signedPreKey,
    preKeyBundle,
    storeContext,
  };
}
