#!/bin/bash

set -e

HOST_OS=$(uname -s)
if [ "$HOST_OS" == "Darwin" ]; then
    HOST_ARCH_S=darwin-x86
else
    HOST_ARCH_S=linux-x86
fi

TARGET_GCC_VER=${TARGET_GCC_VER:-4.9}
TARGET_ARCH=arm-linux-androideabi

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
[target.arm-linux-androideabi]
linker="$GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-$TARGET_GCC_VER/bin/arm-linux-androideabi-gcc"
rustflags = [
  "-C", "link-arg=--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/",
  "-C", "link-arg=-lhidlbase",
  "-C", "link-arg=-lhidltransport",
  "-C", "link-arg=-lutils",
  "-C", "link-arg=-lbinder",
  "-C", "link-arg=-lhwbinder",
  "-C", "link-arg=-lc++",
  "-C", "opt-level=z",
]
EOF
fi

# Use device name as a temp solution to differentiate the build.
if [ -d "$GONK_DIR/prebuilts/ndk/r13" ]; then
NDK_VERSION=r13
else
NDK_VERSION=9
fi

export ANDROID_PLATFORM=android-21
export NDK_INCLUDE_DIR=$GONK_DIR/prebuilts/ndk/$NDK_VERSION/platforms/$ANDROID_PLATFORM/arch-arm/usr/include

# Needed to cross compile C dependencies properly.
export PATH=$GONK_DIR/prebuilts/gcc/$HOST_ARCH_S/arm/arm-linux-androideabi-$TARGET_GCC_VER/bin:$PATH
export PATH=$GONK_DIR/prebuilts/clang/host/linux-x86/clang-4053586/bin/:$PATH
# export CC=arm-linux-androideabi-gcc
export CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$NDK_INCLUDE_DIR"
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

# Use the same Rust toolchain and target as Gecko.
#export RUSTUP_TOOLCHAIN=1.37.0
export RUST_TARGET=arm-linux-androideabi # TODO: switch to thumbv7neon-linux-androideabi

echo "Doing a ${RUST_TARGET} ${TARGET} build with features: [${FEATURES}]"

# This package has a build dep on `backtrace` which get confused by cross compiler settings,
# so we built it alone first.

# And set CFLAGS again for the remaining crates.
#CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/ -I$NDK_INCLUDE_DIR"
CFLAGS="--sysroot=$GONK_DIR/out/target/product/$PRODUCT_NAME/obj/"
CFLAGS="$CFLAGS -mthumb -Os -fomit-frame-pointer -fno-strict-aliasing -fno-exceptions -Wno-multichar -ffunction-sections -fdata-sections -funwind-tables -fstack-protector-strong -Wa,--noexecstack -Werror=format-security -D_FORTIFY_SOURCE=2 -fno-short-enums -no-canonical-prefixes -DNDEBUG -g -Wstrict-aliasing=2 -DANDROID -fmessage-length=0 -W -Wall -Wno-unused -Winit-self -Wpointer-arith -DNDEBUG -D__compiler_offsetof=__builtin_offsetof -Wno-unused-command-line-argument -fdebug-prefix-map=$PWD/= -Werror=non-virtual-dtor -Werror=address -Werror=sequence-point -Werror=date-time -msoft-float -mfloat-abi=softfp -mfpu=neon -mcpu=cortex-a7 -mfpu=neon-vfpv4 -D__ARM_FEATURE_LPAE=1"
CFLAGS="$CFLAGS -I$GONK_DIR/system/libhidl/base/include/ -I$GONK_DIR/system/core/libcutils/include -I$GONK_DIR/system/core/libutils/include -I$GONK_DIR/system/core/libbacktrace/include -I$GONK_DIR/system/core/liblog/include -I$GONK_DIR/system/core/libsystem/include -I$GONK_DIR/system/libhidl/transport/include -I$GONK_DIR/system/core/base/include -I$GONK_DIR/out/soong/.intermediates/system/libhidl/transport/manager/1.0/android.hidl.manager@1.0_genc++_headers/gen -I$GONK_DIR/out/soong/.intermediates/system/libhidl/transport/manager/1.1/android.hidl.manager@1.1_genc++_headers/gen -I$GONK_DIR/out/soong/.intermediates/system/libhidl/transport/base/1.0/android.hidl.base@1.0_genc++_headers/gen -I$GONK_DIR/system/libhwbinder/include -I$GONK_DIR/external/libcxx/include -I$GONK_DIR/external/libcxxabi/include -I$GONK_DIR/system/core/include -I$GONK_DIR/system/media/audio/include -I$GONK_DIR/hardware/libhardware/include -I$GONK_DIR/hardware/libhardware_legacy/include -I$GONK_DIR/hardware/ril/include -I$GONK_DIR/libnativehelper/include -I$GONK_DIR/frameworks/native/include -I$GONK_DIR/frameworks/native/opengl/include -I$GONK_DIR/frameworks/av/include -isystem $GONK_DIR/bionic/libc/arch-arm/include -isystem $GONK_DIR/bionic/libc/include -isystem $GONK_DIR/bionic/libc/kernel/uapi -isystem $GONK_DIR/bionic/libc/kernel/uapi/asm-arm -isystem $GONK_DIR/bionic/libc/kernel/android/scsi -isystem $GONK_DIR/bionic/libc/kernel/android/uapi -I$GONK_DIR/libnativehelper/include_deprecated"
CFLAGS="$CFLAGS -std=c++14 -DANDROID -DANDROID_STRICT -D_USING_LIBCXX -D_LIBCPP_ENABLE_THREAD_SAFETY_ANNOTATIONS -DNDEBUG -D_FORTIFY_SOURCE=2"
CFLAGS="$CFLAGS -Wsign-promo -Wno-inconsistent-missing-override -Wno-null-dereference -D_LIBCPP_ENABLE_THREAD_SAFETY_ANNOTATIONS -Wno-thread-safety-negative -fno-rtti -fvisibility-inlines-hidden -Werror=int-to-pointer-cast -Werror=pointer-to-int-cast"
CFLAGS="$CFLAGS -target $TARGET_ARCH -mcpu=cortex-a7 -msoft-float -mfloat-abi=softfp -mfpu=neon"
#CFLAGS="$CFLAGS -I$GONK_DIR/out/soong/.intermediates/system/libhidl/transport/manager/1.0/android.hidl.manager@1.0_genc++_headers/gen/"
#CFLAGS="$CFLAGS -I$GONK_DIR/out/soong/.intermediates/system/libhidl/transport/base/1.0/android.hidl.base@1.0_genc++_headers/gen/"
export CFLAGS

if [ "$HOST_OS" == "Darwin" ]; then
export CXXFLAGS="$CFLAGS"
export CPPFLAGS="$CFLAGS"
export CC=arm-linux-androideabi-gcc
export CXX=arm-linux-androideabi-g++
fi
export CC=clang
export CXX=clang++
export LD=clang

cargo build ${FEATURES} --target=${RUST_TARGET} ${OPT}

