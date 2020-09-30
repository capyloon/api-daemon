#!/bin/bash

set -e

HOST_OS=$(uname -s)
if [ "$HOST_OS" == "Darwin" ]; then
    HOST_ARCH_S=darwin-x86
else
    HOST_ARCH_S=linux-x86
fi

TARGET_GCC_VER=${TARGET_GCC_VER:-4.9}

export ANDROID_PLATFORM=android-21

# Check that the GONK_DIR environment variable is set
# and build the .cargo/config file from it.
if [ -z ${GONK_DIR+x} ];
then
    echo "Please set GONK_DIR to the root of your Gonk directory first.";
    exit 1;
else
    # Get the product name from .config
    source $GONK_DIR/.config
    CARGO_CONFIG=`pwd`/.cargo/config
    PRODUCT_NAME=${TARGET_NAME:-${PRODUCT_NAME}}
    echo "Using '$GONK_DIR' to create '$CARGO_CONFIG' for '$PRODUCT_NAME'";
    mkdir -p `pwd`/.cargo
    cat << EOF > $CARGO_CONFIG
[target.armv7-linux-androideabi]
linker="$GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-$TARGET_GCC_VER/bin/arm-linux-androideabi-gcc"
rustflags = [
  "-C", "link-arg=--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/",
  "-C", "opt-level=z",
]
EOF
fi

# Needed to cross compile C dependencies properly.
export PATH=$GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-$TARGET_GCC_VER/bin:$PATH
# export CC=arm-linux-androideabi-gcc
export CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$GONK_DIR/prebuilts/ndk/9/platforms/$ANDROID_PLATFORM/arch-arm/usr/include"

export GIT_BUILD_INFO=`git log -n 1 --pretty=format:"%H "; date +%d/%m/%Y-%H:%M:%S`

FEATURES=${FEATURES}
STRIP=
OPT=
TARGET=debug
while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)
            OPT="$OPT --release"
            TARGET=release
            ;;
        --strip)
            STRIP=yes
            ;;
        --no-default-features)
            OPT="$OPT --no-default-features"
            ;;
    esac
    shift
done

echo "Doing a ${TARGET} build with ${FEATURES}"

# unset the sysroot for the `backtrace` build deps so they don't pick up the wrong sysroot. 
unset CFLAGS

# for `libcurl``
export PKG_CONFIG_ALLOW_CROSS=1

if [ "$HOST_OS" == "Darwin" ]; then
export CXXFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$GONK_DIR/prebuilts/ndk/9/platforms/android-21/arch-arm/usr/include"
export CPPFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$GONK_DIR/prebuilts/ndk/9/platforms/android-21/arch-arm/usr/include"
export CC=arm-linux-androideabi-gcc
export CXX=arm-linux-androideabi-g++
fi

# And set CFLAGS again for the remaining crates.
export CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$GONK_DIR/prebuilts/ndk/9/platforms/android-21/arch-arm/usr/include"

cargo build ${FEATURES} --target=armv7-linux-androideabi ${OPT}

DAEMON=../target/armv7-linux-androideabi/${TARGET}/child-test-daemon
# Generate symbols
HOST_OS=$(uname -s)
if [ "$HOST_OS" == "Darwin" ]; then
    DUMP_SYMS=../tools/dump_syms/dump_syms_mac
else
    DUMP_SYMS=../tools/dump_syms/dump_syms
fi
python ../tools/dump_syms/generate_breakpad_symbols.py --dump-syms-dir ../tools/dump_syms \
    --symbols-dir ../target/armv7-linux-androideabi/${TARGET}/kaios --binary $DAEMON

if [ "${STRIP}" = "yes" ];
then
    # Explicitely strip the binary since even release builds have symbols.
    $GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-$TARGET_GCC_VER/bin/arm-linux-androideabi-strip $DAEMON
fi
