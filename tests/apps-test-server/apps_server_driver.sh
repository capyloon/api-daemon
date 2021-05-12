#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

function kill_server() {
    pid=$(ps -ef | grep apps_test_server | grep -v grep | awk '{print $2}');
    if [ -n "$pid" ]; then
        kill -9 $pid;
        echo "Killed apps_test_server (pid $pid)";
    fi
}

# Reset apps
rm -rf $CI_PROJECT_DIR/prebuilts/http_root/webapps/

cd $CI_PROJECT_DIR/tests/apps-test-server

# Align with config-webdriver.toml
rm -rf ../webapps
rm -f $CI_PROJECT_DIR/services/apps/test-fixtures/webapps/apps
ln -s $CI_PROJECT_DIR/services/apps/test-fixtures/webapps ../webapps
ln -s $CI_PROJECT_DIR/services/apps/client $CI_PROJECT_DIR/services/apps/test-fixtures/webapps/apps

$CI_PROJECT_DIR/target/release/apps_test_server &
$CI_PROJECT_DIR/tests/apps-test-server/v1.sh
DONT_CREATE_WEBAPPS=1 $CI_PROJECT_DIR/tests/webdriver.sh http://apps.localhost:8081/test/tests.html
kill_server

$CI_PROJECT_DIR/tests/apps-test-server/v2.sh
$CI_PROJECT_DIR/target/release/apps_test_server &
DONT_CREATE_WEBAPPS=1 $CI_PROJECT_DIR/tests/webdriver.sh http://apps.localhost:8081/test/tests_update_app.html
kill_server

rm ../webapps

echo "WebDriver apps tests success"
