#!/bin/bash

set -x -e

RELEASE_ROOT=./prebuilts/http_root/api/v1
if [ 'Test'$1 != 'Test' ]
  then
RELEASE_ROOT=$1
fi

echo "Release libs to ${RELEASE_ROOT}"

cd test-service/client/
yarn install
yarn prod
cd ../..

cd common/client/
yarn install
yarn prodsession
yarn prodcore
cd ../..

mkdir -p ${RELEASE_ROOT}/shared
mkdir -p ${RELEASE_ROOT}/test

cp ./common/client/dist/core.js ./common/client/dist/session.js  ${RELEASE_ROOT}/shared/
cp ./test-service/client/dist/service.js ${RELEASE_ROOT}/test/

find ${RELEASE_ROOT} -name "*.gz" -delete
gzip --recursive --best --force ${RELEASE_ROOT}
