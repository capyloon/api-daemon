#!/bin/bash

set -e

if [ -z ${CI_PROJECT_DIR+x} ];
then
    echo "Please set CI_PROJECT_DIR to the path of your SIDL repository.";
    exit 1;
fi

if [ -z ${CI_ACCESS_TOKEN+x} ];
then
    echo "Please set CI_ACCESS_TOKEN to a valid token value.";
    exit 1;
fi

# Download and unpack the latest b2g desktop.
pushd ${CI_PROJECT_DIR}/tests > /dev/null

echo "Downloading b2g.linux-x86_64.tar.bz2 to `pwd`"

url="${url_prefix}/releng/ci-util/b2g-desktop.git"
git clone ${url} -b next --depth=1 ./b2g-desktop
echo
git -C ./b2g-desktop log -1
tar -xf ./b2g-desktop/b2g.linux-x86_64.tar.bz2 -C ${GIT_CLONE_PATH}/tests
