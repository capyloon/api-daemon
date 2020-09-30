
#include "libsignal-protocol-c/src/protocol.h"
#include "libsignal-protocol-c/src/signal_protocol.h"
#include "libsignal-protocol-c/src/key_helper.h"
#include "libsignal-protocol-c/src/signal_protocol_internal.h"
#include "libsignal-protocol-c/src/session_builder.h"
#include "libsignal-protocol-c/src/session_cipher.h"
#include "libsignal-protocol-c/src/group_session_builder.h"
#include "libsignal-protocol-c/src/group_cipher.h"

/**
  * Some types definitions are hidden in .c files to only expose
  * opaque structs in headers. We need to copy them here to
  * use them trough FFI.
  */

/* Copied from libsignal-protocol-c/src/curve.c */

#ifdef __cplusplus
extern "C" {
#endif

#define DJB_KEY_LEN 32

struct ec_public_key
{
    signal_type_base base;
    uint8_t data[DJB_KEY_LEN];
};

struct ec_private_key
{
    signal_type_base base;
    uint8_t data[DJB_KEY_LEN];
};

struct ec_key_pair
{
    signal_type_base base;
    ec_public_key *public_key;
    ec_private_key *private_key;
};

/* Copied from libsignal-protocol-c/src/ratchet.c */

struct ratchet_identity_key_pair
{
    signal_type_base base;
    ec_public_key *public_key;
    ec_private_key *private_key;
};

/* Copied from libsignal-protocol-c/src/session_pre_key.c */

struct session_pre_key
{
    signal_type_base base;
    uint32_t id;
    ec_key_pair *key_pair;
};

struct session_signed_pre_key
{
    signal_type_base base;
    uint32_t id;
    ec_key_pair *key_pair;
    uint64_t timestamp;
    size_t signature_len;
    uint8_t signature[];
};

struct session_pre_key_bundle
{
    signal_type_base base;
    uint32_t registration_id;
    int device_id;
    uint32_t pre_key_id;
    ec_public_key *pre_key_public;
    uint32_t signed_pre_key_id;
    ec_public_key *signed_pre_key_public;
    signal_buffer *signed_pre_key_signature;
    ec_public_key *identity_key;
};

/* Copied from libsignal-protocol-c/src/keyhelper.c */
struct signal_protocol_key_helper_pre_key_list_node
{
    session_pre_key *element;
    struct signal_protocol_key_helper_pre_key_list_node *next;
};

/* Copied from libsignal-protocol-c/src/protocol.c */
struct ciphertext_message
{
    signal_type_base base;
    int message_type;
    signal_context *global_context;
    signal_buffer *serialized;
};

struct signal_message
{
    ciphertext_message base_message;
    uint8_t message_version;
    ec_public_key *sender_ratchet_key;
    uint32_t counter;
    uint32_t previous_counter;
    signal_buffer *ciphertext;
};

struct pre_key_signal_message
{
    ciphertext_message base_message;
    uint8_t version;
    uint32_t registration_id;
    int has_pre_key_id;
    uint32_t pre_key_id;
    uint32_t signed_pre_key_id;
    ec_public_key *base_key;
    ec_public_key *identity_key;
    signal_message *message;
};

struct sender_key_distribution_message
{
    ciphertext_message base_message;
    uint32_t id;
    uint32_t iteration;
    signal_buffer *chain_key;
    ec_public_key *signature_key;
};

/* Implemented in helpers.c */
void ciphertext_message_destroy(ciphertext_message *message);

#ifdef __cplusplus
}
#endif
