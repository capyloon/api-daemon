/**
 * A session is responsible for doing the initial handshake
 * and tracking requests.
 * It hides the specificity of the transport layer.
 */

import WSTransport from "./ws_transport";
import services from "./services";

const DEBUG = false;

export class Session {
  constructor() {
    // Mapping of request ids to Promises that will be resolved
    // or rejected.
    this.requests = new Map();
    // Start at 1 to only generate odd request numbers.
    this.request_id = 1;
    this.next_id = 1;
    this.tracked = new Map();
    this.tracked_events = new Map();
    this.connected = false;
    this.reconnecting = false;
  }

  /**
   * Open a session using a given url.
   * the status of the transportation will be notified via onsessionconnected
   * and onsessiondisconnected callbacks
   *
   * @param {string} url - the url to connect to, should start with ws:// or wss://
   * @param {string} token - the token used to get accese to service.
   *                 If navigator.b2g.externalapi is available, then we use the token
   *                 from navigator.b2g.externalapi.getToken() as default. Else, we use
   *                 token to get access to service.
   * @param {object} session_state - it has three callbacks, onsessionconnected,
   *                 onsessiondisconnected and onsessionconnectionfailed. They
   *                 are called when session is connected, disconnected when previously
   *                 connected, and connectionfailed when previous session is not setup.
   * @param {boolean} lazy_reconnect - Lazy reopen connections if daemon crashed,
   *                  Try to re-connected daemon if lazy_reconnect is true.
   *                  If connected, onsessionconnected will be called.
   *                  other wise, wait for daemon is up and session is connected.
   *
   * @return  return object {success: true|false, cause: "..." }
   */
  open_url(url, token, session_state, lazy_reconnect = false) {
    let ret = {
      success: false,
      cause: "error",
    };

    if (!url.startsWith("ws://") && !url.startsWith("wss://")) {
      ret.success = false;
      ret.cause = `Unsupported url. Url should be a valid websocket url`;
      return ret;
    }

    this.session_state = session_state;
    this.lazy_reconnect = lazy_reconnect;
    this.url = url;
    this.token = token;

    let ws_start = function (session) {
      session.transport = new WSTransport();
      session.transport.set_listener(session);
      url = url || "ws://localhost:8081/ws";
      session.transport.start(url, session.token);
      ret.success = true;
      ret.cause = "success";
    };

    if (navigator.b2g && navigator.b2g.externalapi) {
      navigator.b2g.externalapi.getToken().then((token) => {
        //console.log("JS Client tcpsocket get token is " + token);
        this.token = token;
        ws_start(this);
      });
    } else {
      ws_start(this);
    }

    return ret;
  }

  /**
   * Open a session using a given transport.
   * Valid values for transport are: "websocket".
   *
   * @param {string} name - the name of transport, only websocket is available.
   * @param {string} host - the host ip address
   *
   * This is a wrapper around open_url, for other parameters description
   * and return value see open_url function.
   */
  open(name, host, token, session_state, lazy_reconnect = false) {
    if (name !== "websocket") {
      ret.success = false;
      ret.cause = `Unsupported transport: ${name}. Valid values are: "websocket"`;

      return ret;
    }

    host = host || "localhost:8081";

    return this.open_url(`ws://${host}/ws`, token, session_state, lazy_reconnect);
  }

  /**
   * Close the session explicitly. When it's successfully closed, onsessiondisconnected
   * of the session_state will be called.
   *
   */
  close() {
    this.lazy_reconnect = false;
    let socket = this.transport.socket;
    if (socket) {
      socket.close();
    }
    this.clear("client closed session");
  }

  /**
   * Clear tracked objects and reject all the promises when session is closed.
   *
   * @param {string} description - the description to reject all the requests.
   */
  clear(description) {
    this.requests.forEach((value, key, map) => {
      value.reject({
        reason: "session_closed",
        value: description,
      });
    });
    this.requests.clear();

    this.tracked.forEach((value, key, map) => {
      value = null;
      map.delete(key);
    });
    this.tracked = null;

    this.tracked_events.forEach((value, key, map) => {
      value = null;
      map.delete(key);
    });
    this.tracked_events = null;

    this.transport = null;
  }

  /**
   * Get the state of session.
   *
   * @return true means connected, false for disconnected.
   */
  is_connected() {
    return this.connected;
  }

  /**
   * Re-open a session using a given transport. Refreshes all the state of Session
   * the status of the transportation will be notified via onsessionconnected
   * and onsessiondisconnected callbacks, which is set in open()
   *
   */
  reconnect() {
    this.next_id = 1;
    this.tracked = new Map();
    this.tracked_events = new Map();
    this.transport = new WSTransport();
    let ret = {
      success: false,
      cause: "error",
    };
    this.transport.set_listener(this);
    var url = this.url || "ws://localhost:8081";
    this.transport.start(url, this.token);
    ret.success = true;
    ret.cause = "success";

    return ret;
  }

  /**
   * get next id.
   *
   * @return {string} next id
   *
   */
  get_next_id() {
    return this.next_id;
  }

  /**
   * track object
   *
   * @param {string} obj - the object to be tracked
   *
   */
  track(obj) {
    this.tracked.set(this.next_id, obj);
    this.next_id = this.next_id + 2;
  }

  /**
   * untrack object
   *
   * @param {string} id - the id used to untrack mapped object
   *
   */
  untrack(id) {
    this.tracked.delete(id);
  }

  /**
   * get tracked object by id
   *
   * @param {string} id - the id mapped to object
   *
   */
  get_tracked(id) {
    if (this.tracked.has(id)) {
      return this.tracked.get(id);
    } else {
      return null;
    }
  }

  /**
   * Creates a new tracked object.
   *
   * @param {class} obj_class    - the class to be tracked.
   * @param {any}   optional_arg - an argument to pass to the constructor.
   *
   */
  new_tracked(obj_class, optional_arg) {
    let obj = new obj_class(this.next_id, this, optional_arg);
    this.track(obj);
    return obj;
  }

  /**
   * track object which is for receiving events
   *
   * @param {integer} service_id - service id for each service
   * @param {integer} object_id - object id represents object
   * @param {string} obj - the object to be tracked
   *
   */
  track_events(service_id, object_id, obj) {
    let key = this.get_event_key(service_id, object_id);
    if (!this.tracked_events.has(key)) {
      this.tracked_events.set(key, obj);
    }
  }

  /**
   * untrack object for events
   *
   * @param {integer} service_id - service id for each service
   * @param {integer} object_id - object id represents object
   *
   */
  untrack_events(service_id, object_id) {
    let key = this.get_event_key(service_id, object_id);
    this.tracked_events.delete(key);
  }

  /**
   * get tracked events object by service_id and object_id
   *
   * @param {integer} service_id - service id for each service
   * @param {integer} object_id - object id represents object
   *
   */
  get_tracked_events(service_id, object_id) {
    let key = this.get_event_key(service_id, object_id);
    if (this.tracked_events.has(key)) {
      return this.tracked_events.get(key);
    } else {
      return null;
    }
  }

  /**
   * Compose a list use service_id object_id as key.
   *
   * @param {integer} service_id - service id for each service
   * @param {integer} object_id - object id represents object
   *
   * @return {string} key - composed key
   */
  get_event_key(service_id, object_id) {
    let comma = ",";
    let key = service_id + comma + object_id;

    return key;
  }

  /**
   * Dispatch to the right service/object/promise once we have decoded the common header.
   * If request_id is in our request map, resolve the promise
   *
   * @param {binary} data - the data send from low level implementation to be decoded
   *
   */
  on_message(data) {
    // Dispatch to the right service/object/promise once we have decoded the common header.
    // If request_id is in our request map, resolve the promise
    let message = ExternalAPI.core.BaseMessage.decode(data);
    let kind = message.kind;
    DEBUG &&
      console.log(
        `Session message: kind=${kind} service=${message.service} obj=${message.object}`
      );
    if (DEBUG && kind === 1) {
      console.log(`got response #${message.response}`);
    }
    if (kind === 1 && this.requests.has(message.response)) {
      // Response to a request we sent.
      let ctxt = this.requests.get(message.response);
      // Unpack the object based on the ctxt.response_type.
      let msg = ctxt.response_type.decode(
        message.content,
        message.service,
        this
      );
      let full = {
        response: message.response,
        service: message.service,
        object: message.object,
        msg: msg,
      };
      this.requests.delete(message.response);
      ctxt.resolve(full);
    } else if (kind === 0) {
      // Requests to a callback object.
      // We are not tracking this request.
      // Look if this object id is one expected to process messages itself.
      DEBUG &&
        console.log(
          `Message for service ${message.service}, callback object ${message.object}`
        );

      // This is targeted at a tracked object, use the listener for this object.
      let listener = this.get_tracked(message.object);
      if (listener && listener.on_message) {
        DEBUG && console.log(`dispatching message to %o`, listener);
        listener.on_message(message);
      } else {
        console.error(
          `No object available (id ${message.object}) to process this request: %o`,
          message
        );
      }
    } else if (kind === 2) {
      // Event message.
      // Check if that's a message for an event listener.
      DEBUG &&
        console.log(
          `Looking for listeners for events on service #${message.service} object #${message.object}`
        );
      let event_listener = this.get_tracked_events(
        message.service,
        message.object
      );
      if (event_listener && event_listener.on_event) {
        DEBUG &&
          console.log(
            `dispatching event_listener message to %o`,
            event_listener
          );
        event_listener.on_event(message.content);

        return;
      }
    } else if (kind === 3) {
      // Permission error
      let error = message.permissionError;
      // Cancel the request.
      let ctxt = this.requests.get(error.request);
      if (ctxt) {
        ctxt.reject({
          reason: "permission_error",
          value: error,
        });
        this.requests.delete(error.request);
      }
    }
  }

  /**
   *
   * @return {u32} next id usable for requests numbers.
   */
  next_id() {
    let res = this.next_id + 2;
    this.next_id = res;
    res;
  }

  /**
   * Send a request, and unwrap the actual expected response type.
   *
   * @param {string} message - message to be encoded.
   * @param {string} response_type - response type which is used to decode response message
   * @return {Promise} the promise resolved when the response type is received.
   */
  send_request(base_message, response_type) {
    // If the session is not connected anymore but we still try to send a request,
    // return a rejected promise.
    if (!this.connected) {
      return Promise.reject({
        reason: "session_closed",
        value: `zombie session (message was ${JSON.stringify(base_message)})`,
      });
    }

    if (!response_type) {
      return Promise.reject({
        reason: "call_failure",
        value: `response_type must be defined (message was ${JSON.stringify(
          base_message
        )}).`,
      });
    }

    this.request_id = this.request_id + 2;
    base_message.set_request(this.request_id);

    let buffer = base_message.encode();
    this.transport.send(buffer);

    return new Promise((resolve, reject) => {
      this.requests.set(this.request_id, {
        resolve,
        reject,
        response_type,
      });
    });
  }

  /**
   * Send a request that has no matching response (like member setters).
   *
   * @param {string} message - message to be encoded.
   * @param {string} response_type - response type which is used to decode response message
   * @return void.
   */
  send_request_oneway(base_message) {
    // If the session is not connected anymore but we still try to send a request,
    // throw and error
    if (!this.connected) {
      throw new Error(
        `session_closed: zombie session (message was ${JSON.stringify(
          base_message
        )})`
      );
    }

    this.request_id = this.request_id + 2;
    base_message.set_request(this.request_id);

    let buffer = base_message.encode();
    this.transport.send(buffer);
  }

  /**
   *
   * @param {object} base_message - the message to encode and send.
   */
  send_callback_message(base_message) {
    // If the session is not connected anymore but we still try to send a request,
    // throw and error
    if (!this.connected) {
      throw new Error(
        `session_closed: zombie session (message was ${JSON.stringify(
          base_message
        )})`
      );
    }

    this.transport.send(base_message.encode());
  }

  /**
   * Report connection to Session user.
   * Let user know error happens, and try to get new Session
   *
   * @param {string} state - the status of the transportation connection.
   *                         The possible states are "connected", "error", "closed"
   *
   */
  set_transport_state(state) {
    switch (state.toLowerCase()) {
      case "connected":
        this.connected = true;
        this.reconnecting = false;
        if (typeof this.session_state.onsessionconnected === "function") {
          this.session_state.onsessionconnected();
        }
        break;
      case "error":
      case "closed":
        // Previously connected, we see it as disconnected,
        // otherwise, we see it connectionfailed
        // test previouse conncted state here to trigger callbacks
        if (this.connected) {
          if (typeof this.session_state.onsessiondisconnected === "function") {
            this.connected = false;
            this.session_state.onsessiondisconnected();
          }
        } else {
          if (
            typeof this.session_state.onsessionconnectionerror === "function"
          ) {
            this.session_state.onsessionconnectionerror();
          }
        }
        if (this.lazy_reconnect && !this.reconnecting) {
          this.clear("daemon restarting");
          this.reconnecting = true;
          this.reconnect_interval = 500;
          this.wait_reopen();
        }
        break;
    }
  }

  wait_reopen() {
    setTimeout(() => {
      if (this.connected) {
        return;
      }
      // Get a new token if possible.
      if (navigator.b2g && navigator.b2g.externalapi) {
        navigator.b2g.externalapi.getToken().then(
          (token) => {
            this.token = token;
            this.reconnect();
          },
          (e) => {
            this.wait_reopen();
          }
        );
      } else {
        this.reconnect();
      }
    }, () => {
      this.reconnect_interval = this.reconnect_interval * 2;
      if (this.reconnect_interval > 8000) {
        return 8000;
      }
      return this.reconnect_interval;
    });
  }

  has_service(name) {
    return services.has(name, this);
  }
}
