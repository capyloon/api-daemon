#!/bin/bash

set -x -e
pwd=`pwd`
cd $pwd/../../../common/client/
yarn install
cd $pwd
