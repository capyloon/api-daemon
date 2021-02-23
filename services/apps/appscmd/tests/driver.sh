#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

cd $CI_PROJECT_DIR/daemon
RUST_LOG=info WS_RUNTIME_TOKEN=secrettoken $CI_PROJECT_DIR/target/release/api-daemon &
export RUST_LOG=debug
export RUST_BACKTRACE=1

cd $CI_PROJECT_DIR/services/apps/appscmd/tests

# Let the daemon start and initialize.
sleep 5

$CI_PROJECT_DIR/target/release/appscmd --socket /tmp/apps_service_uds.sock --json list > apps_observed.json

$CI_PROJECT_DIR/tests/kill_daemon.sh

md5sum apps_expected.json | sed s/expected/observed/ | md5sum -c
