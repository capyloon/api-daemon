#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

# Prepare fota workspace
cd $CI_PROJECT_DIR/services/fota

TEST_SERVER_PORT=10095
if [[ "$1" == "http://fota.localhost:8081/test/tests_server_error.html" ]]; then
TEST_SERVER_PORT=10096
elif [[ "$1" == "http://fota.localhost:8081/test/tests_no_package.html" ]]; then
TEST_SERVER_PORT=10097
elif [[ "$1" == "http://fota.localhost:8081/test/tests_check_update.html" ]]; then
TEST_SERVER_PORT=10098
fi

KOTAJSON=$CI_PROJECT_DIR/services/fota/test-fixtures/kota-$TEST_SERVER_PORT.json
CONFIGJSON=$CI_PROJECT_DIR/services/fota/test-fixtures/config-$TEST_SERVER_PORT.json

rm -rf ./test-workspace
mkdir ./test-workspace
cp $KOTAJSON ./test-workspace/kota.json
cp $CONFIGJSON ./test-workspace/config.json

$CI_PROJECT_DIR/tests/webdriver.sh $@

echo "WebDriver fota test successful for $@"
