#!/bin/bash

set -e

#export RUSTUP_TOOLCHAIN=1.63

export BUILD_APPSCMD=yes
export OSX_CROSS=/home/capyloon/dev/capyloon/osx-cross
export SDKROOT=${OSX_CROSS}/MacOSX11.3.sdk

STRIP=${HOME}/.mozbuild/clang/bin/llvm-strip

function build_target() {
    rm -rf prebuilts/
    mkdir -p prebuilts/${TARGET_ARCH}

    # Build sqlx-macros and its dependencies as a workspace level host crate.
    cargo build --release --target=${TARGET_ARCH} -p sqlx-macros
    
    pushd daemon
    cargo build --release --target=${TARGET_ARCH}
    popd

    ./release_libs.sh
    
    cp target/${TARGET_ARCH}/release/api-daemon prebuilts/${TARGET_ARCH}/
    ${STRIP} prebuilts/${TARGET_ARCH}/api-daemon
    
    pushd services/apps/appscmd
    cargo build --release --target=${TARGET_ARCH}
    popd
    cp target/${TARGET_ARCH}/release/appscmd prebuilts/${TARGET_ARCH}/
    ${STRIP} prebuilts/${TARGET_ARCH}/appscmd
    
    tar cJf api-daemon-${TARGET_ARCH}.tar.xz prebuilts
}

function apple_build() {
    rm -rf prebuilts/
    mkdir -p prebuilts/${TARGET_ARCH}
    ./update-prebuilts.sh

    ${OSX_CROSS}/cctools/bin/${TARGET_ARCH}-strip prebuilts/${TARGET_ARCH}/api-daemon
    ${OSX_CROSS}/cctools/bin/${TARGET_ARCH}-strip prebuilts/${TARGET_ARCH}/appscmd

    tar cJf api-daemon-${TARGET_ARCH}.tar.xz prebuilts
}

# x86_64 desktop build
TARGET_ARCH=x86_64-unknown-linux-gnu
build_target

# Apple aarch64 build
export TARGET_ARCH=aarch64-apple-darwin
apple_build

# Apple x86_64 build
export TARGET_ARCH=x86_64-apple-darwin
apple_build

# Mobian aarch64 build
export MOZBUILD=$HOME/.mozbuild
export TARGET_ARCH=aarch64-unknown-linux-gnu
rm -rf prebuilts/
mkdir -p prebuilts/${TARGET_ARCH}
./update-prebuilts.sh

${STRIP} prebuilts/${TARGET_ARCH}/api-daemon
${STRIP} prebuilts/${TARGET_ARCH}/appscmd

tar cJf api-daemon-${TARGET_ARCH}.tar.xz prebuilts

