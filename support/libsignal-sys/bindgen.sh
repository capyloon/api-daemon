#!/bin/bash

set -e -x

echo "Generating bindings for target $TARGET"

mkdir -p src/generated

INCPATH="-I ./libsignal-protocol-c/src/"

if [ "${TARGET}" != "x86_64-unknown-linux-gnu" ];
then
    INCPATH="${INCPATH} -I ${SYS_INCLUDE_DIR}"
fi

bindgen --whitelist-function "signal_context_create" \
        --whitelist-function "signal_context_destroy" \
        --whitelist-function "signal_context_set_crypto_provider" \
        --whitelist-function "signal_context_set_log_function" \
        --whitelist-function "signal_context_set_locking_functions" \
        --whitelist-function "signal_protocol_key_helper_generate_registration_id" \
        --whitelist-function "signal_protocol_key_helper_generate_identity_key_pair" \
        --whitelist-function "signal_protocol_key_helper_generate_pre_keys" \
        --whitelist-function "signal_protocol_key_helper_key_list_free" \
        --whitelist-function "signal_protocol_key_helper_generate_signed_pre_key" \
        --whitelist-function "ratchet_identity_key_pair_create" \
        --whitelist-function "ratchet_identity_key_pair_destroy" \
        --whitelist-function "session_pre_key_destroy" \
        --whitelist-function "signal_protocol_key_helper_generate_sender_signing_key" \
        --whitelist-function "ec_key_pair_destroy" \
        --whitelist-function "signal_protocol_key_helper_generate_sender_key" \
        --whitelist-function "signal_buffer_create" \
        --whitelist-function "signal_buffer_free" \
        --whitelist-function "signal_protocol_key_helper_generate_sender_key_id" \
        --whitelist-function "signal_protocol_store_context_create" \
        --whitelist-function "signal_protocol_store_context_set_session_store" \
        --whitelist-function "signal_protocol_store_context_set_pre_key_store" \
        --whitelist-function "signal_protocol_store_context_set_signed_pre_key_store" \
        --whitelist-function "signal_protocol_store_context_set_identity_key_store" \
        --whitelist-function "signal_protocol_store_context_set_sender_key_store" \
        --whitelist-function "signal_protocol_store_context_destroy" \
        --whitelist-function "session_builder_create" \
        --whitelist-function "session_builder_process_pre_key_bundle" \
        --whitelist-function "session_builder_free" \
        --whitelist-function "session_pre_key_bundle_create" \
        --whitelist-function "session_pre_key_bundle_destroy" \
        --whitelist-function "curve_generate_key_pair" \
        --whitelist-function "ec_key_pair_get_public" \
        --whitelist-function "session_cipher_create" \
        --whitelist-function "session_cipher_free" \
        --whitelist-function "session_cipher_encrypt" \
        --whitelist-function "session_cipher_decrypt_pre_key_signal_message" \
        --whitelist-function "session_cipher_decrypt_signal_message" \
        --whitelist-function "session_cipher_set_decryption_callback" \
        --whitelist-function "session_cipher_get_remote_registration_id" \
        --whitelist-function "ratchet_identity_key_pair_destroy" \
        --whitelist-function "ciphertext_message_destroy" \
        --whitelist-function "pre_key_signal_message_deserialize" \
        --whitelist-function "pre_key_signal_message_destroy" \
        --whitelist-function "signal_message_deserialize" \
        --whitelist-function "signal_message_destroy" \
        --whitelist-function "curve_calculate_agreement" \
        --whitelist-function "curve_verify_signature" \
        --whitelist-function "curve_calculate_signature" \
        --whitelist-function "curve_decode_point" \
        --whitelist-function "signal_int_list_alloc" \
        --whitelist-function "signal_int_list_push_back" \
        --whitelist-function "group_session_builder_create" \
        --whitelist-function "group_session_builder_free" \
        --whitelist-function "group_session_builder_create_session" \
        --whitelist-function "group_session_builder_process_session" \
        --whitelist-function "sender_key_distribution_message_deserialize" \
        --whitelist-function "sender_key_distribution_message_destroy" \
        --whitelist-function "ciphertext_message_get_serialized" \
        --whitelist-function "sender_key_message_deserialize" \
        --whitelist-function "sender_key_message_destroy" \
        --whitelist-function "group_cipher_create" \
        --whitelist-function "group_cipher_free" \
        --whitelist-function "group_cipher_encrypt" \
        --whitelist-function "group_cipher_decrypt" \
        --whitelist-function "group_cipher_set_decryption_callback" \
        --whitelist-type "ciphertext_message" \
        --whitelist-type "pre_key_signal_message" \
        --whitelist-type "signal_message" \
        --whitelist-type "signal_type_base" \
        --whitelist-type "signal_protocol_key_helper_pre_key_list_node" \
        --whitelist-type "ec_public_key" \
        --whitelist-type "ec_private_key" \
        --whitelist-type "ec_key_pair" \
        --whitelist-type "session_pre_key" \
        --whitelist-type "sender_key_distribution_message" \
        --blacklist-type "session_signed_pre_key" \
        --blacklist-type "signal_buffer" \
        --blacklist-type "__uint8_t" \
        --blacklist-type "__int32_t" \
        --blacklist-type "__uint32_t" \
        --blacklist-type "__uint64_t" \
        --output src/generated/ffi.rs \
        --no-layout-tests \
        --with-derive-default \
        wrapper.h \
        -- ${INCPATH}

cat custom-types.rs >> src/generated/ffi.rs
