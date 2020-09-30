// Manages the JS side of the core objects.

import { Encoder, ObjectDecoder, Decoder } from "./bincode.js";

const DEBUG = false;

export const core = (() => {
  class SessionHandshake {
    constructor(version, token) {
      this.version = version;
      this.token = token;
    }

    encode() {
      let encoder = new Encoder();
      return encoder.string(this.version).string(this.token).value();
    }
  }

  class SessionAck {
    constructor() {}

    decode(buffer) {
      return new ObjectDecoder(buffer).bool("success").finalize();
    }
  }

  class BaseMessage {
    // Creates a base message. At this point we don't
    // know yet the request value because it's set by the
    // session when the base message is about to be sent.
    constructor(service, object, content) {
      this.service = service;
      this.object = object;
      this.content = content;
    }

    set_request(request) {
      this.kind = 0;
      this.request = request;
    }

    set_response(response) {
      this.kind = 1;
      this.response = response;
    }

    is_event() {
      this.kind = 2;
      this.is_event = true;
    }

    encode() {
      if (!this.content) {
        throw Error("No content available");
      }

      let encoder = new Encoder();
      encoder = encoder.u32(this.service).u32(this.object);

      if (this.request !== undefined) {
        DEBUG && console.log(`Encoding request #${this.request}`);
        encoder = encoder.enum_tag(0).u64(this.request);
      } else if (this.response !== undefined) {
        encoder = encoder.enum_tag(1).u64(this.response);
      } else if (this.is_event) {
        DEBUG && console.log(`Encoding event`);
        encoder = encoder.enum_tag(2);
      } else {
        throw Error("Unable to serialize message: kind is not set properly");
      }

      return encoder.u8_array(this.content).value();
    }

    static decode(buffer) {
      let decoder = new Decoder(buffer);
      let message = { service: decoder.u32() };

      message.object = decoder.u32();
      let kind = decoder.enum_tag();

      if (kind === 0) {
        message.request = decoder.u64();
      } else if (kind === 1) {
        message.response = decoder.u64();
      } else if (kind === 2) {
        message.is_event = true;
      } else if (kind === 3) {
        let error = {
          request: decoder.u64(),
        };
        error.permission = decoder.string();
        error.message = decoder.string();
        message.permissionError = error;
      } else {
        throw Error(`Unexpected kind of message: ${kind}`);
      }
      message.kind = kind;
      message.content = decoder.u8_array();
      return message;
    }
  }

  const CoreRequest = {
    // Encode the variant matching the params.
    encode: (params) => {
      if (params.variant === CoreRequest.GET_SERVICE) {
        let encoder = new Encoder();
        return encoder
          .enum_tag(params.variant)
          .string(params.name)
          .string(params.fingerprint)
          .value();
      } else if (params.variant === CoreRequest.HAS_SERVICE) {
        let encoder = new Encoder();
        return encoder.enum_tag(params.variant).string(params.name).value();
      } else if (
        params.variant === CoreRequest.ENABLE_EVENT ||
        params.variant === CoreRequest.DISABLE_EVENT
      ) {
        let encoder = new Encoder();
        return encoder
          .enum_tag(params.variant)
          .u32(params.service)
          .u32(params.object)
          .u32(params.event)
          .value();
      } else if (params.variant === CoreRequest.RELEASE_OBJECT) {
        let encoder = new Encoder();
        return encoder
          .enum_tag(params.variant)
          .u32(params.service)
          .u32(params.object)
          .value();
      } else {
        console.error(`Unknown variant: ${params.variant}`);
      }
    },

    GET_SERVICE: 0,
    HAS_SERVICE: 1,
    RELEASE_OBJECT: 2,
    ENABLE_EVENT: 3,
    DISABLE_EVENT: 4,
  };

  const CoreResponse = {
    decode: (buffer) => {
      let decoder = new Decoder(buffer);
      let variant = decoder.enum_tag();
      if (variant === CoreRequest.GET_SERVICE) {
        // Decode a GetService response.
        let success = decoder.bool();
        let service = decoder.u32();
        return { variant: CoreRequest.GET_SERVICE, success, service };
      } else if (variant === CoreRequest.HAS_SERVICE) {
        // Decode a GetService response.
        let success = decoder.bool();
        let service = decoder.u32();
        return { variant: CoreRequest.HAS_SERVICE, success, service };
      } else if (variant === CoreRequest.RELEASE_OBJECT) {
        // Decode a ReleaseObject response.
        return { variant: CoreRequest.RELEASE_OBJECT, success: decoder.bool() };
      } else if (variant === CoreRequest.ENABLE_EVENT) {
        // Decode a EnableEvent response.
        return { variant: CoreRequest.ENABLE_EVENT, success: decoder.bool() };
      } else if (variant === CoreRequest.DISABLE_EVENT) {
        // Decode a DisableEvent response.
        return { variant: CoreRequest.DISABLE_EVENT, success: decoder.bool() };
      } else {
        throw Error(`Unknown variant: ${variant}`);
      }
    },
  };

  return {
    SessionAck,
    SessionHandshake,
    BaseMessage,
    CoreRequest,
    CoreResponse,
  };
})();
