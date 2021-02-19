/**
 * Base object of a Session service object.
 */

import services from "../../../common/client/src/services";

var kcore = ExternalAPI.core;

class SessionObject {
  /**
   * Initialize a SessionObject object.
   * @constructor
   * @param {string} id - object id.
   * @param {Session} session - the service session to send function calls.
   * @param {object} service - the service which represents a context
   * @param {string} wrapper_payload_message - the payload message for each different
   * service.
   *
   */
  constructor(id, session, service, wrapper_payload_message) {
    this.id = id;
    this.session = session;
    this.service_id = service ? service.id : 0;
    this.wrapper_payload_message = wrapper_payload_message;
    this.event_callbacks = Object.create(null);
  }

  /**
   * Send a request, and unwrap the actual expected response type.
   *
   * @param {string} message - message to be encoded.
   * @param {string} response_type - response type which is used to decode response message
   *
   */
  send_request(message, response_type) {
    // let prefix = `request-${response_type}-${this.id}-`;
    //window.performance.mark(`${prefix}start`);
    return this.session
      .send_request(message, this.wrapper_payload_message[response_type])
      .then((res) => {
        // console.log(`response: ${JSON.stringify(res)}`);
        //window.performance.measure(`${prefix}duration`, `${prefix}start`);
        // let entry = performance.getEntriesByName(`${prefix}duration`, "measure");
        // console.log(`Request took ${entry[0].duration.toFixed(2)}ms`);
        let msg = res.msg;
        if (msg.success !== undefined) {
          return msg.success;
        } else {
          // console.error(`>> rejecting ${response_type} %o`, msg[response_type].error);
          return Promise.reject({
            reason: "call_failure",
            value: msg.error,
          });
        }
      })
      .catch((error) => {
        return Promise.reject(error);
      });
  }

  /**
   * Send a request that has no matching response (like member setters).
   *
   * @param {string} message - message to be encoded.
   *
   */
  send_request_oneway(message) {
    return this.session.send_request_oneway(message);
  }

  /**
   *
   * @param {object} base_message - the message to encode and send.
   */
  send_callback_message(base_message) {
    let message = new kcore.BaseMessage(
      base_message.service,
      base_message.object,
      base_message.content
    );
    message.set_response(base_message.request);
    this.session.send_callback_message(message);
  }

  /**
   * Do function calls.
   *
   * @param {string} method_name - method name.
   * @param {object} payload - parameter for the function call
   *
   */
  call_method(method_name, payload) {
    // console.log(`SessionObject call_method ${method_name} ${JSON.stringify(payload || "")}`);
    let buff = this.create_payload_message(`${method_name}Request`, payload);
    let message = new kcore.BaseMessage(this.service_id, this.id, buff);
    return this.send_request(message, `${method_name}Response`).then(
      (res) => {
        return res;
      },
      (error) => {
        // Only assign method name if error is an object (cf.`send_request`)
        if (error && error.reason === "call_failure") {
          const e = Object.assign({ method_name: method_name }, error);
          return Promise.reject(e);
        }

        if (error && error.reason === "permission_error") {
          const e = Object.assign({ method_name: method_name }, error);
          // Remove internal routing value.
          delete e.value.request;
          console.error(
            `Permission error in ${method_name}: missing '${e.value.permission}' - ${e.value.message}`
          );
          return Promise.reject(e);
        }

        return Promise.reject(error);
      }
    );
  }

  /**
   * Do function calls.
   *
   * @param {string} method_name - method name.
   * @param {object} payload - parameter for the function call
   *
   */
  call_method_oneway(method_name, payload) {
    // console.log(`SessionObject call_method_oneway ${method_name} ${JSON.stringify(payload || "")}`);
    let buff = this.create_payload_message(`${method_name}Request`, payload);
    let message = new kcore.BaseMessage(this.service_id, this.id, buff);
    this.send_request_oneway(message);
  }

  /**
   * Release current object when it's no longer used.
   *
   */
  release() {
    let buff = services.create_core_message(kcore.CoreRequest.RELEASE_OBJECT, {
      service: this.service_id,
      object: this.id,
    });
    let message = new kcore.BaseMessage(0, 0, buff);
    return this.session
      .send_request(message, kcore.CoreResponse)
      .then((res) => {
        console.log("CoreResponse is %o", res);
        if (
          res.msg.variant === kcore.CoreRequest.RELEASE_OBJECT &&
          res.msg.success
        ) {
          return Promise.resolve();
        }

        return Promise.reject({
          reason: "call_failure",
          value: `No such object`,
        });
      });
  }

  /**
   * Enable event.
   *
   * @param {integer} event_id - event id which is defined in each service
   *
   */
  enable_event(event_id) {
    let buff = services.create_core_message(kcore.CoreRequest.ENABLE_EVENT, {
      service: this.service_id,
      object: this.id,
      event: event_id,
    });
    let message = new kcore.BaseMessage(0, 0, buff);
    this.session.send_request(message, kcore.CoreResponse);
  }

  /**
   * Disable event.
   *
   * @param {integer} event_id - event id which is defined in each service
   *
   */
  disable_event(event_id) {
    let buff = services.create_core_message(kcore.CoreRequest.DISABLE_EVENT, {
      service: this.service_id,
      object: this.id,
      event: event_id,
    });
    let message = new kcore.BaseMessage(0, 0, buff);
    this.session.send_request(message, kcore.CoreResponse);
  }

  /**
   * Compose a list use service_id object_id and event_id as key to store callbacks.
   *
   * @param {integer} event_id - event id which is defined in each service
   * @return {string} key - composed key
   */
  get_event_key(event_id) {
    let comma = ",";
    let key = this.service_id + comma + this.id + comma + event_id;

    return key;
  }

  /**
   * Add event listener for event of each services.
   *
   * @param {integer} event_id - event id which is defined in each service
   * @param {function} callback - function to call when event received
   *
   */
  addEventListener(event_id, callback) {
    const key = this.get_event_key(event_id);
    if (!Object.prototype.hasOwnProperty.call(this.event_callbacks, key)) {
      this.event_callbacks[key] = [];
      this.enable_event(event_id);
    } else if (this.event_callbacks[key].includes(callback)) {
      // Duplicated listeners are not allowed.
      return;
    }

    this.event_callbacks[key].push(callback);
  }

  /**
   * Remove event listener for event of each services.
   *
   * @param {integer} event_id - event id which is defined in each service
   * @param {function} callback - function to stop receiving events
   *
   */
  removeEventListener(event_id, callback) {
    const key = this.get_event_key(event_id);
    if (!Object.prototype.hasOwnProperty.call(this.event_callbacks, key)) {
      return;
    }

    // Get rid of matched callbacks.
    this.event_callbacks[key] = this.event_callbacks[key].filter(
      (element) => element !== callback
    );

    // Delete the entry as well as disable the event when
    // no callbacks available.
    if (this.event_callbacks[key].length === 0) {
      delete this.event_callbacks[key];
      this.disable_event(event_id);
    }
  }

  /**
   * Dispatch event.
   *
   * @param {integer} event_id - event id which is defined in each service
   * @param {binary} data - the data passed to listener
   *
   */
  dispatchEvent(event_id, data) {
    const key = this.get_event_key(event_id);
    const stack = this.event_callbacks[key];
    if (stack === undefined) {
      return;
    }
    stack.forEach((element) => {
      element.call(this, data);
    });
  }

  /**
   * Create payload message based on wrapperPayloadMessage.
   * This function performs the wrapping in a wrapperPayloadMessage message
   *
   * @param {string} kind - the kind of functions.
   * @param {object} init_value - paramater value for the function call
   *
   */
  create_payload_message(variant, init_value) {
    // console.log(`SessionObject create_payload_message ${variant} ${JSON.stringify(init_value || "")}`);

    return this.wrapper_payload_message[variant].encode(init_value || {});
  }
}

export { SessionObject as default };
