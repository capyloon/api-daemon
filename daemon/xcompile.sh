#!/bin/bash

set -e
source ../utils.sh
FEATURES=${FEATURES}
STRIP=
OPT=
BUILD_TYPE=debug
while [[ $# -gt 0 ]]; do
    case "$1" in
    --release)
        OPT="$OPT --release"
        BUILD_TYPE=release
        ;;
    --strip)
        STRIP=yes
        ;;
    --no-default-features)
        OPT="$OPT --no-default-features"
        ;;
    esac
    shift
done

echo "Doing a ${BUILD_TYPE} build with ${FEATURES}"

setup_xcompile_envs
xcompile
binary=../target/${TARGET_ARCH}/${BUILD_TYPE}/api-daemon
generate_breakpad_symbols ${binary}
if [ "${STRIP}" = "yes" ]; then
    xstrip ${binary}
fi
