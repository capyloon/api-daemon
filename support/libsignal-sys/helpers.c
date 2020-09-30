/* Custom functions that are not provided directly by libsignal */

#include "libsignal-protocol-c/src/signal_protocol.h"
#include "libsignal-protocol-c/src/signal_protocol_types.h"

void ciphertext_message_destroy(ciphertext_message* message) {
    SIGNAL_UNREF(message);
}