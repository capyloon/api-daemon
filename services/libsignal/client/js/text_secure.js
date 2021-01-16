// Protobuf encoder for the TextSecure (pre)-key structures.
//
//======================= begin protobuf description ============
// package textsecure;
//
// message PreKeyRecordStructure {
//     optional uint32 id        = 1;
//     optional bytes  publicKey = 2;
//     optional bytes  privateKey = 3;
// }
//
// message SignedPreKeyRecordStructure {
//     optional uint32  id         = 1;
//     optional bytes   publicKey  = 2;
//     optional bytes   privateKey = 3;
//     optional bytes   signature  = 4;
//     optional fixed64 timestamp  = 5;
// }
//======================= end protobuf description =============
//
// Because of how simple they are, we use a custom encoder instead of a
// full fledged protobuf implementation.
// See https://developers.google.com/protocol-buffers/docs/encoding

const WIRE_TYPE_VARINT = 0;
const WIRE_TYPE_64BITS = 1;
const WIRE_TYPE_BYTES = 2;

const Protobuf = {
  // Because fieldNumber <= 5 here, we know this will never be more than 42,
  // so we don't need a varint encoding for the tag itself.
  tag(wireType, fieldNumber) {
    return (fieldNumber << 3) | wireType;
  },

  uint32Size(value) {
    let len =
      value < 128
        ? 1
        : value < 16384
        ? 2
        : value < 2097152
        ? 3
        : value < 268435456
        ? 4
        : 5;
    return len;
  },

  // Returns the array holding the representation of this uint32.
  uint32(value, buffer, pos) {
    while (value > 127) {
      buffer[pos++] = (value & 127) | 128;
      value >>>= 7;
    }
    buffer[pos] = value;

    return pos + 1;
  },

  // Returns { hi:..., lo:... }
  split64(value) {
    if (value === 0) return { hi: 0, lo: 0 };
    let sign = value < 0;
    if (sign) value = -value;
    let lo = value >>> 0,
      hi = ((value - lo) / 4294967296) >>> 0;
    if (sign) {
      hi = ~hi >>> 0;
      lo = ~lo >>> 0;
      if (++lo > 4294967295) {
        lo = 0;
        if (++hi > 4294967295) hi = 0;
      }
    }
    return { hi, lo };
  },

  fixed32At(value, buffer, pos) {
    buffer[pos] = value & 255;
    buffer[pos + 1] = (value >>> 8) & 255;
    buffer[pos + 2] = (value >>> 16) & 255;
    buffer[pos + 3] = value >>> 24;
  },

  fixed64(value, buffer, pos) {
    const { hi, lo } = Protobuf.split64(value);
    Protobuf.fixed32At(lo, buffer, pos);
    Protobuf.fixed32At(hi, buffer, pos + 4);
    return pos + 8;
  },

  // Bytes are serialized as len(uint32) + the bytes.
  bytes(value, buffer, pos) {
    let length = value.byteLength;
    pos = Protobuf.uint32(length, buffer, pos);
    buffer.set(value, pos);
    return pos + length;
  },

  bytesSize(array) {
    let length = array.byteLength;
    return Protobuf.uint32Size(length) + length;
  },
};

class TextSecure {
  _createPubKeyBuffer(publicKey) {
    // The public key needs an extra byte in front when serialized...
    let pubKeyBuffer = new Uint8Array(33);
    pubKeyBuffer[0] = 0x05;
    pubKeyBuffer.set(publicKey, 1);
    return pubKeyBuffer;
  }

  // preKey is a SessionSignedPreKey
  serializeSignedPreKey(preKey) {
    let pubKeyBuffer = this._createPubKeyBuffer(preKey.keyPair.publicKey);

    let size =
      5 + // 5 bytes for the tags, and add each field size.
      Protobuf.uint32Size(preKey.id) +
      Protobuf.bytesSize(pubKeyBuffer) +
      Protobuf.bytesSize(preKey.keyPair.privateKey) +
      Protobuf.bytesSize(preKey.signature) +
      8; // 8 bytes for the fixed64 timestamp.

    let buffer = new Uint8Array(size);
    let pos = 0;

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_VARINT, 1);
    pos = Protobuf.uint32(preKey.id, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_BYTES, 2);
    pos = Protobuf.bytes(pubKeyBuffer, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_BYTES, 3);
    pos = Protobuf.bytes(preKey.keyPair.privateKey, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_BYTES, 4);
    pos = Protobuf.bytes(preKey.signature, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_64BITS, 5);
    pos = Protobuf.fixed64(preKey.timestamp, buffer, pos);

    return buffer;
  }

  // preKey is a SessionPreKey
  serializePreKey(preKey) {
    let pubKeyBuffer = this._createPubKeyBuffer(preKey.keyPair.publicKey);

    let size =
      3 + // 3 bytes for the tags, and add each field size.
      Protobuf.uint32Size(preKey.id) +
      Protobuf.bytesSize(pubKeyBuffer) +
      Protobuf.bytesSize(preKey.keyPair.privateKey);

    let buffer = new Uint8Array(size);
    let pos = 0;

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_VARINT, 1);
    pos = Protobuf.uint32(preKey.id, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_BYTES, 2);
    pos = Protobuf.bytes(pubKeyBuffer, buffer, pos);

    buffer[pos++] = Protobuf.tag(WIRE_TYPE_BYTES, 3);
    pos = Protobuf.bytes(preKey.keyPair.privateKey, buffer, pos);

    return buffer;
  }
}
