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

curl --header "PRIVATE-TOKEN: $CI_ACCESS_TOKEN" https://git.kaiostech.com/api/v4/projects/11099/repository/files/b2g.linux-x86_64.tar.bz2/raw?ref=next -o b2g.linux-x86_64.tar.bz2
tar xf b2g.linux-x86_64.tar.bz2
popd > /dev/null
