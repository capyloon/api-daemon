/**
 * Helper to get a reference to a service.
 */

import { core } from "./core.js";

const DEBUG = false;

const Services = {
  /**
   * Retrieves a service, returning a Promise with the service id.
   *
   * @param {string} name - the name of the service to create.
   * @param {string} fingerprint - the fingerprint of the service.
   * @param {Session} session - the service session to send function calls.
   * @return {Promise} resolve service id
   *
   */
  get: (name, fingerprint, session) => {
    DEBUG && console.log(`Services.get(${name})`);
    let buff = core.CoreRequest.encode({
      variant: core.CoreRequest.GET_SERVICE,
      name: name,
      fingerprint: fingerprint,
    });

    let base_message = new core.BaseMessage(
      /*service*/ 0,
      /*object*/ 0,
      /*content*/ buff
    );
    return session
      .send_request(base_message, core.CoreResponse)
      .then((res) => {
        DEBUG &&
          console.log(`Service.get(${name}) response: ${JSON.stringify(res)}`);
        let msg = res.msg;
        if (
          msg.variant === core.CoreRequest.GET_SERVICE &&
          msg.response.service
        ) {
          return msg.response.service;
        }

        return Promise.reject(msg.response.error);
      });
  },

  /**
   * Checks if the named service is available.
   *
   * @param {string} name - the name of the service to check.
   * @param {Session} session - the service session to send the call.
   * @return {Promise} resolve to true or false.
   *
   */
  has: (name, session) => {
    DEBUG && console.log(`Services.has(${name})`);
    let buff = core.CoreRequest.encode({
      variant: core.CoreRequest.HAS_SERVICE,
      name: name,
    });

    let message = new core.BaseMessage(
      /*service*/ 0,
      /*object*/ 0,
      /*content*/ buff
    );
    return session.send_request(message, core.CoreResponse).then((res) => {
      DEBUG &&
        console.log(`Service.has(${name}) response: ${JSON.stringify(res)}`);
      if (res.msg.success !== undefined) {
        return res.msg.success;
      }

      return Promise.reject(`hasService failed for: ${name}`);
    });
  },

  /**
   * Creates a core message of the given kind and initial value.
   * This function performs the wrapping in a PayloadMessage message.
   *
   * @param {string} kind - the kind of functions.
   * @param {object} init_value - paramater value for the function call
   *
   */
  create_core_message: (variant, init_value) => {
    DEBUG &&
      console.log(
        `Services create_core_message ${variant} ${JSON.stringify(init_value)}`
      );
    // null/undefined properties are not set at all which means
    // that oneof sets end up being empty. So we set it to an empty
    // object instead.
    if (!init_value) {
      init_value = {};
    }

    init_value.variant = variant;
    return core.CoreRequest.encode(init_value);
  },
};

export { Services as default };
