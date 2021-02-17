#!/bin/bash

set -x -e
BUILD_TYPE=${BUILD_TYPE:-prod}
RELEASE_ROOT=${RELEASE_ROOT:-./prebuilts/http_root/api/v1}
echo "Release libs to ${RELEASE_ROOT}"

. utils.sh

if [ 'Test'$WITH_TEST_SERVICE != 'Test' ]; then
# We are running tests.
    mkdir -p ./prebuilts/http_root/tests/fixtures/
    cp ./tests/testing.js ./prebuilts/http_root/tests/testing.js
    cp ./tests/testing.css ./prebuilts/http_root/tests/testing.css
    cp -R ./tests/fixtures ./prebuilts/http_root/tests/
    cp ./services/libsignal/test-fixtures/example3 ./prebuilts/http_root/tests/fixtures/
    release_service_lib test ${RELEASE_ROOT} ${BUILD_TYPE}
fi

release_service_lib apps ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib audiovolumemanager ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib contacts ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib contentmanager ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib devicecapability ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib libsignal ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib powermanager ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib procmanager ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib settings ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib tcpsocket ${RELEASE_ROOT} ${BUILD_TYPE}
release_service_lib time ${RELEASE_ROOT} ${BUILD_TYPE}

release_shared_lib ${RELEASE_ROOT} ${BUILD_TYPE}
