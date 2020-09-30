#!/bin/bash

set -x -e

if [ -z ${UPDATER_DIR+x} ];
then
    echo "Please set UPDATER_DIR to your updater repo first.";
    exit 1;
fi

if [ -z ${BUILD_VARIANT+x} ];
then
    echo "Please set BUILD_VARIANT=[user|userdebug|eng]";
    exit 1;
fi

if [ "${CUSTOM_BUILD_CONFIG_NAME}" = "rjil" ]; then
DEVICE_CONFIG_NAME=config-device-${CUSTOM_BUILD_CONFIG_NAME}
else
DEVICE_CONFIG_NAME=config-device
fi

if [ "${BUILD_VARIANT}" = "user" ]; then
config_file=daemon/${DEVICE_CONFIG_NAME}-production.toml
else
config_file=daemon/${DEVICE_CONFIG_NAME}-dev.toml
fi

PATH=$UPDATER_DIR/target/release:$UPDATER_DIR/target/debug:$PATH

# copy resources we want to ship to a temporary directory.
mkdir -p dist
cp ${config_file} dist/config.toml
cp prebuilts/armv7-linux-androideabi/api-daemon dist/
cp -R prebuilts/http_root dist/

RUST_LOG=info packagebuilder --src dist/ --mount service/api-daemon

mv /tmp/api-daemon*.zip .
mv /tmp/api-daemon*.toml .

rm -rf dist/
