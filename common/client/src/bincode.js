// A bincode encoder / decoder toolkit for JS.
// See https://github.com/bincode-org/bincode

// This library provides a way to decode and encode primitive types,
// and is expected to be used by higher level libraries to offer a
// full roundtrip Rust <-> JS.

// All methods are synchronous and return the success value. If an
// error is encountered, an exception is thrown.

// zigzag encoding.
function zigzag_encode(val) {
  const i32 = 0x100000000;
  // get integeral value
  let valH = Math.floor(val / i32);
  // treat lower 32 bits as unsiged
  let valL = (val & 0xffffffff) >>> 0;

  // The least significant bit(LSB) of encodedH is the most significant bit(MSB) of valL
  let encodedH = (valL >>> 31) ^ (valH << 1) ^ (valH >> 31);
  // lower bits are unsigned
  let encodedL = ((valL << 1) ^ (valH >> 31)) >>> 0;
  return encodedH * i32 + encodedL;
}

function zigzag_decode(val) {
  const i32 = 0x100000000;
  var valH = Math.floor(val / i32);
  var valL = (val & 0xffffffff) >>> 0;

  let decodedH = (valH >>> 1) ^ -(valL & 1);
  // * The MSB of decodedL is the LSB of valH
  // * lower bits are unsiged
  let decodedL = ((valH << 31) ^ (valL >>> 1) ^ -(valL & 1)) >>> 0;
  return decodedH * i32 + decodedL;
}

export class Decoder {
  constructor(buffer) {
    this.reset(buffer);
  }

  reset(buffer) {
    this.buffer = buffer;
    this.pos = 0;
  }

  // Throws if we decoded the whole buffer.
  throw_if_eof() {
    if (this.pos >= this.buffer.length) {
      throw Error("eof");
    }
  }

  // Throws if less than N bytes are still readable.
  throw_if_less_than(num) {
    if (this.pos + num - 1 >= this.buffer.length) {
      throw Error("eof");
    }
  }

  // Read a varint encoded integer.
  varint() {
    this.throw_if_less_than(1);

    let val = this.buffer[this.pos];
    this.pos += 1;

    if (val <= 250) {
      return val;
    }

    // u16
    if (val == 251) {
      return this.full_u16();
    }

    // u32
    if (val == 252) {
      return this.full_u32();
    }

    // u64
    if (val == 253) {
      return this.full_u64();
    }

    if (val == 254) {
      // No support for 128bits integers.
      throw Error("Failed to read varint: 128bits integers ar not supported.");
    }

    throw Error(`Failed to read varint: unexpected byte: ${val}`);
  }

  // Read a void. This is a no-op but helps with code generation using the decoder
  // since it's very common for return types.
  void() {
    return {};
  }

  // Boolean are 0u8=false 1u8=true
  bool() {
    this.throw_if_less_than(1);
    let val = this.buffer[this.pos];
    this.pos += 1;
    return val === 1;
  }

  // Strings are encoded as:
  // u64: len
  // len bytes for the ut8 encoded string.
  string() {
    let len = this.varint();
    this.throw_if_less_than(len);

    let utf8 = this.buffer.subarray(this.pos, this.pos + len);
    this.pos += len;
    let decoder = new TextDecoder("utf-8");
    return decoder.decode(utf8);
  }

  // Read a JSON representation of an object.
  json() {
    return JSON.parse(this.string());
  }

  // Read a u64
  // Note that JS range for integers is only -(2^53 - 1) .. (2^53 - 1)
  full_u64() {
    this.throw_if_less_than(8);

    let val = 0;
    for (let i = 0; i < 8; i++) {
      val = val * 256 + this.buffer[this.pos];
      this.pos += 1;
    }
    return val;
  }

  u64() {
    return this.varint();
  }

  // Read a i64
  // Note that JS range for integers is only -(2^53 - 1) .. (2^53 - 1)
  i64() {
    return zigzag_decode(this.u64());
  }

  // Read a Vec<u8> or [u8]
  u8_array() {
    let len = this.varint();

    this.throw_if_less_than(len);
    let data = this.buffer.subarray(this.pos, this.pos + len);
    this.pos += len;
    return data;
  }

  // Read a u16
  full_u16() {
    this.throw_if_less_than(2);

    let val = 0;
    for (let i = 0; i < 2; i++) {
      val = (val << 8) + this.buffer[this.pos];
      this.pos += 1;
    }
    return val;
  }

  // Read a u32
  full_u32() {
    this.throw_if_less_than(4);

    let val = 0;
    for (let i = 0; i < 4; i++) {
      // if the MSB of val is on, the last '<< 8' causes val turns to negative.
      // >>> 0 to make sure we get unsigned value
      val = ((val << 8) >>> 0) + this.buffer[this.pos];
      this.pos += 1;
    }
    return val;
  }

  u32() {
    return this.varint();
  }

  // Read an enum tag
  enum_tag() {
    return this.varint();
  }

  // Read a Date as a i64 milliseconds since epoch
  date() {
    let date = new Date();
    date.setTime(this.i64());
    return date;
  }
}

function checkType(val, type) {
  let ok = false;
  switch (type) {
    case "uintarray":
      ok = typeof val.BYTES_PER_ELEMENT === "number";
      break;
    case "date":
      ok = typeof val.toGMTString === "function";
      break;
    default:
      ok = typeof val === type;
  }
  if (!ok) {
    let msg = `Expected ${type}, bug got ${typeof val}`;
    let e = new Error(msg);
    console.error(`======================= Start Bincode Type Checking error =======================`);
    console.error(msg);
    console.error(e.stack);
    console.error(`=======================  End Bincode Type Checking error  =======================`);
    throw e;
  }
}

export class Encoder {
  constructor() {
    this.reset();
  }

  reset() {
    // Initial buffer size, we'll grow it as needed.
    this.buffer = new Uint8Array(64);
    this.size = 0;
  }

  // Extend the current buffer if needed to make sure we
  // can store num bytes.
  extend_if_needed(num) {
    if (this.buffer.length - this.size < num) {
      let new_buffer = new Uint8Array(this.size + num * 2);
      new_buffer.set(this.buffer);
      this.buffer = new_buffer;
    }
  }

  // Write a varint value.
  varint(val) {
    checkType(val, "number");

    // We will always write at least one byte: either the value
    // or the tag for the value range.
    this.extend_if_needed(1);

    if (val <= 250) {
      this.buffer[this.size] = val;
      this.size += 1;
      return this;
    }

    // 251 -> u16 max value
    if (val < 1 << 16) {
      this.buffer[this.size] = 251;
      this.size += 1;
      return this.full_u16(val);
    }

    // 252 -> u32 max value
    if (val < 1 << 32) {
      this.buffer[this.size] = 252;
      this.size += 1;
      return this.full_u32(val);
    }

    // 253 -> u64 max value
    this.buffer[this.size] = 253;
    this.size += 1;
    return this.full_u64(val);
  }

  // Write an enum tag
  enum_tag(val) {
    return this.varint(val);
  }

  // Write a void. This is a no-op but helps with code generation using the encoder.
  void(val) {
    return this;
  }

  // Write a u16
  full_u16(val) {
    checkType(val, "number");

    this.extend_if_needed(2);
    this.size += 2;

    for (let i = 0; i < 2; i++) {
      this.buffer[this.size - 1 - i] = val & 0xff;
      val >>= 8;
    }
    return this;
  }

  // Write a u32
  full_u32(val) {
    checkType(val, "number");

    this.extend_if_needed(4);
    this.size += 4;

    for (let i = 0; i < 4; i++) {
      this.buffer[this.size - 1 - i] = val & 0xff;
      val >>= 8;
    }
    return this;
  }

  // Write a u64
  full_u64(val) {
    checkType(val, "number");

    this.extend_if_needed(8);

    this.size += 8;
    for (let i = 0; i < 8; i++) {
      this.buffer[this.size - 1 - i] = val & 0xff;
      val /= 256;
    }
    return this;
  }

  // Write a i64
  i64(val) {
    this.u64(zigzag_encode(val));
    return this;
  }

  u64(val) {
    this.varint(val);
    return this;
  }

  u32(val) {
    this.varint(val);
    return this;
  }

  // Write a string
  string(val) {
    checkType(val, "string");

    // First encode the string as utf-8
    let encoder = new TextEncoder();
    // Send the utf-8 array.
    return this.u8_array(encoder.encode(val));
  }

  // Write an object as json, by stringifying.
  json(obj) {
    return this.string(JSON.stringify(obj));
  }

  // Write a boolean
  bool(val) {
    this.extend_if_needed(1);
    this.buffer[this.size] = !!val ? 1 : 0;
    this.size += 1;
    return this;
  }

  // Write a Vec<u8>
  u8_array(val) {
    checkType(val, "uintarray");

    this.varint(val.length);
    this.extend_if_needed(val.length);
    this.buffer.set(val, this.size);
    this.size += val.length;
    return this;
  }

  // Write a Date as i64 milliseconds since epoch.
  date(val) {
    checkType(val, "date");

    this.i64(val.getTime());
    return this;
  }

  // Returns the final buffer without the trailing bytes
  // that could have been allocated when growing the buffer.
  value() {
    return this.buffer.slice(0, this.size);
  }
}

let variant = {
  bool: (name) => {
    return () => {
      return { name: name, kind: "_bool" };
    };
  },

  i64: (name) => {
    return () => {
      return { name: name, kind: "_i64" };
    };
  },

  u64: (name) => {
    return () => {
      return { name: name, kind: "_u64" };
    };
  },

  u32: (name) => {
    return () => {
      return { name: name, kind: "_u32" };
    };
  },

  string: (name) => {
    return () => {
      return { name: name, kind: "_string" };
    };
  },

  u8_array: (name) => {
    return () => {
      return { name: name, kind: "_u8_array" };
    };
  },
};

export function ObjectDecoder(buffer) {
  // console.log(`Creating ObjectDecoder`);
  this.decoder = new Decoder(buffer);
  this.value = {};

  this.bool = (name) => {
    this.value[name] = this.decoder.bool();
    return this;
  };

  this._bool = () => {
    return this.decoder.bool();
  };

  this.string = (name) => {
    this.value[name] = this.decoder.string();
    return this;
  };

  this.json = (name) => {
    this.value[name] = this.decoder.json();
    return this;
  };

  this._string = () => {
    return this.decoder.string();
  };

  this.u32 = (name) => {
    this.value[name] = this.decoder.u32();
    return this;
  };

  this._u32 = () => {
    return this.decoder.u32();
  };

  this.u64 = (name) => {
    this.value[name] = this.decoder.u64();
    return this;
  };

  this._u64 = () => {
    return this.decoder.u64();
  };

  this.i64 = (name) => {
    this.value[name] = this.decoder.i64();
    return this;
  };

  this._i64 = () => {
    return this.decoder.i64();
  };

  this.u8_array = (name) => {
    this.value[name] = this.decoder.u8_array();
    return this;
  };

  this._u8_array = () => {
    this.decoder.u8_array();
  };

  (this.enum = (name, variants) => {
    // Create the container for the enum value.
    this.value[name] = {};
    let tag = this.decoder.enum_tag();
    let res = variants[tag]();
    this.value[name][res.name] = this[res.kind](res.name);
    return this;
  }),
    (this.finalize = () => {
      return this.value;
    });
}

function test_decoder() {
  let encoder = new Encoder();

  // String: "Allons à la plage"
  let input = new Uint8Array([
    0, 0, 0, 0, 0, 0, 0, 18, 65, 108, 108, 111, 110, 115, 32, 195, 160, 32, 108,
    97, 32, 112, 108, 97, 103, 101,
  ]);
  let decoder = new Decoder(input);
  console.log(`string: ${decoder.string()}`);

  console.log(`${encoder.string("Allons à la plage").value()}`);

  // i64: -4567
  decoder.reset(new Uint8Array([255, 255, 255, 255, 255, 255, 238, 41]));
  console.log(`i64: ${decoder.i64()}`);

  // i64: 7892
  decoder.reset(new Uint8Array([0, 0, 0, 0, 0, 0, 30, 212]));
  console.log(`i64: ${decoder.i64()}`);

  encoder.reset();
  encoder.i64(7892);
  console.log(`encoded i64: ${encoder.value()}`);

  encoder.reset();
  encoder.i64(-4567);
  console.log(`encoded i64: ${encoder.value()}`);

  // Vec<u8>
  decoder.reset(new Uint8Array([0, 0, 0, 0, 0, 0, 0, 5, 1, 2, 3, 4, 5]));
  console.log(`Vec<u8>: ${decoder.u8_array()}`);

  // Deserialize this Rust struct:
  // enum Choice {
  //     A(i64),
  //     B(String),
  // }

  // struct Demo {
  //     ready: bool,
  //     desc: String,
  //     version: i64,
  //     content: Choice,
  // }

  let result = new ObjectDecoder(
    new Uint8Array([
      1, 0, 0, 0, 0, 0, 0, 0, 14, 68, 195, 169, 109, 111, 110, 115, 116, 114,
      97, 116, 105, 111, 110, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 7, 227,
    ])
  )
    .bool("ready")
    .string("desc")
    .i64("version")
    .enum("content", [variant.i64("A"), variant.string("B")])
    .finalize();

  console.log("Result is %o", result);
}

// test_decoder();
