#!/bin/bash

set -e -x

echo "Generating bindings for target $TARGET"

mkdir -p src/generated

INCPATH=""

if [ "${TARGET}" = "armv7-linux-androideabi" ];
then
    INCPATH="-I ${SYS_INCLUDE_DIR}"
fi

bindgen --output src/generated/selinux_ffi.rs \
        --whitelist-function "setexeccon" \
        --whitelist-function "getexeccon" \
        --whitelist-function "security_getenforce" \
        --whitelist-function "security_load_policy" \
        --no-layout-tests \
        selinux.h \
        -- ${INCPATH}
