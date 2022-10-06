
/**
 * WebSocket transport.
 * Takes care of managing the initial handshake.
 */

import { core } from "./core.js";

class WSTransport {
    /**
     * Initialize a web socket transport object.
     * @constructor
     * @param {WebSocket} socket - WebSocket object.
     * @param {string} state - the state of the WSTransport
     * @param {object} listener - listener for websocket status.
     *
     */
    constructor() {
        this.socket = null;
        // Can be "start", "request", "failed".
        this.state = "start";
        this.listener = null;
    }

    /**
     * Start kaios services. The status of the socket connection will be returned by calling
     * set_transport_state.
     *
     * @param {string} url - the url to start service.
     *
     *
     */
    start(url, token) {
        this.socket = new WebSocket(url);
        this.socket.binaryType = "arraybuffer";
        this.state = "start";

        this.socket.onopen = () => {
            console.log("Websocket: opened");
            // Send the handshake
            let handshake = new core.SessionHandshake(token);
            this.socket.send(handshake.encode());
        }

        this.socket.onmessage = (event) => {
            if (this.state == "start") {
                // We are waiting for the session Ack.
                let ack = new core.SessionAck().decode(new Uint8Array(event.data));
                console.log(`Got ack: ${JSON.stringify(ack)}`);
                if (!ack.success) {
                    this.socket.close();
                    if (this.listener) {
                        this.listener.set_transport_state("closed");
                    }
                } else {
                    this.state = "request";
                    if (this.listener) {
                        this.listener.set_transport_state("connected");
                    }
                }
            } else if (this.state == "request") {
                if (this.listener) {
                    this.listener.on_message(new Uint8Array(event.data));
                }
            }
        }

        this.socket.onerror = (event) => {
            console.log("Websocket: error " + event);
            this.state = "failed";
            if (this.listener) {
                this.listener.set_transport_state("error");
            }
            this.socket.close();
        }

        this.socket.onclose = (event) => {
            console.log("Websocket: close code " + event.code);
            this.socket = null;
            this.state = "failed";
            if (this.listener) {
                this.listener.set_transport_state("closed");
            }
        }
    }

    /**
     * Set listener to receive data from low layer service.
     *
     * @param {object} val - object which wants to receive data from low layer service.
     *
     */
    set_listener(val) {
        this.listener = val;
    }

    /**
     * Send data.
     *
     * @param {binary} buffer - data to be sent.
     *
     */
    send(buffer) {
        if (this.state == "request") {
            this.socket.send(buffer);
        }
    }
}

export { WSTransport as default };
