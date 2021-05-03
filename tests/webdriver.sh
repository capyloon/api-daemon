#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

# Kill child processes on exit.
trap 'jobs -p | xargs kill' EXIT

if [ -z ${DONT_CREATE_WEBAPPS} ];
then
    # Cleanup webapps directory.
    rm -rf ${CI_PROJECT_DIR}/prebuilts/http_root/webapps

    # Check for each service if a service/xyz/client/manifest.webmanifest file
    # is present, and if so add it to the application list.
    WEBAPPS_JSON=${CI_PROJECT_DIR}/tests/webapps/webapps.json
    mkdir -p ${CI_PROJECT_DIR}/tests/webapps
    rm -rf ${CI_PROJECT_DIR}/tests/webapps/*
    echo "[" > ${WEBAPPS_JSON}
    pushd ${CI_PROJECT_DIR}/services > /dev/null
    for service in `ls -d *`; do
        if [ -f ${service}/client/manifest.webmanifest ];
        then
            echo "Registering ${service} tests"
            echo "{ \"name\": \"${service}\", \"manifest_url\": \"http://${service}.localhost:8081/manifest.webmanifest\" }," >> ${WEBAPPS_JSON}
            ln -s `realpath ${service}/client/` ${CI_PROJECT_DIR}/tests/webapps/${service}
        fi
    done

    popd > /dev/null
    # Add an extra entry to make the file valid Json.
    echo "{ \"name\": \"dummy\", \"manifest_url\": \"http://example.com\" }]" >> ${WEBAPPS_JSON}
fi

export FIREFOX_BIN="${CI_PROJECT_DIR}/tests/b2g/b2g"
pushd ${CI_PROJECT_DIR}/daemon > /dev/null
RUST_LOG=debug ${CI_PROJECT_DIR}/target/release/api-daemon ${CI_PROJECT_DIR}/daemon/config-webdriver.toml &
rm -rf ./tmp-profile
mkdir -p ./tmp-profile/webapps
export TEST_FIREFOX_PROFILE=${CI_PROJECT_DIR}/daemon/tmp-profile
export RUST_LOG=info
export RUST_BACKTRACE=1
${CI_PROJECT_DIR}/target/release/driver 2>/dev/null $@
rm -rf ./tmp-profile
popd > /dev/null

echo "==========================================================="
echo "WebDriver test successful for $@"
echo "==========================================================="
