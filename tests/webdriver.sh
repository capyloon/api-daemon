#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

cd $CI_PROJECT_DIR/daemon
RUST_LOG=info WS_RUNTIME_TOKEN=secrettoken $CI_PROJECT_DIR/target/release/api-daemon &
rm -rf ./tmp-profile
mkdir ./tmp-profile
export TEST_FIREFOX_PROFILE=$CI_PROJECT_DIR/daemon/tmp-profile
export RUST_LOG=info
export RUST_BACKTRACE=1
$CI_PROJECT_DIR/target/release/driver $1
rm -rf ./tmp-profile
$CI_PROJECT_DIR/tests/kill_daemon.sh

sleep 5
echo "WebDriver test successful for $1"
