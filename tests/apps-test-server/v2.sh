#!/bin/bash

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

cp ${CI_PROJECT_DIR}/tests/apps-test-server/apps/ciautotest/manifest.webmanifest_v2 ${CI_PROJECT_DIR}/tests/apps-test-server/apps/ciautotest/manifest.webmanifest
cp ${CI_PROJECT_DIR}/tests/apps-test-server/apps/ciautotest/app1_v2.zip  ${CI_PROJECT_DIR}/tests/apps-test-server/apps/ciautotest/application.zip
cp ${CI_PROJECT_DIR}/tests/apps-test-server/apps/pwa/manifest.webmanifest_v2 ${CI_PROJECT_DIR}/tests/apps-test-server/apps/pwa/manifest.webmanifest
