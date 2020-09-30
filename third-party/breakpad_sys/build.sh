#!/bin/bash
set -e

if [ -z "${C_LIBRARY_DIR}" ] || [ -z "${CC_BUILD_DIR}" ];
then
    echo "This script is supposed to be run by \`build.rs\`!" >&2
    exit 1
fi

cmd() {
    echo " • Running: $* …" >&2
    "$@"
}

if test $(uname) = "Linux"; then
    if ! [ -e "${CC_BUILD_DIR}/.git" ];
    then
        cmd git clone https://chromium.googlesource.com/breakpad/breakpad ./${CC_BUILD_DIR} 
        cmd git clone https://chromium.googlesource.com/linux-syscall-support ./${CC_BUILD_DIR}/src/third_party/lss
    fi
    if ! [ -d "src/generated" ];
    then 
        cmd mkdir src/generated
    fi
fi

cmd cd "${CC_BUILD_DIR}"

if [ "${TARGET}" = "armv7-linux-androideabi" ];
then
    #patch for fix compile conflict in wchar.h
    cmd rm -f src/common/android/testing/include/wchar.h

    if [ -z "${GONK_DIR}" ];
    then
        echo "Please set GONK_DIR to the root of your Gonk directory first.";
        exit 1;
    else
        source $GONK_DIR/.config
    fi
    SYSROOT="${GONK_DIR}/prebuilts/ndk/9/platforms/android-21/arch-arm"
    export CC="$GONK_DIR/prebuilts/gcc/linux-x86/arm/arm-linux-androideabi-4.9/bin/arm-linux-androideabi-gcc"
    export CFLAGS="-D__ANDROID__  --sysroot=$SYSROOT"
    export CXX="$GONK_DIR/prebuilts/gcc/linux-x86/arm/arm-linux-androideabi-4.9/bin/arm-linux-androideabi-gcc"
    export CXXFLAGS="-fpermissive -D__ANDROID__  --sysroot=$SYSROOT\
                    -I${GONK_DIR}/external/libcxx/include"
    cmd ./configure --host=arm-linux-androideabi \
                --disable-processor \
                --disable-tools \
                --includedir="${GONK_DIR}/prebuilts/ndk/9/platforms/android-21/arch-arm/usr/include"
else
    CC=cc
    CFLAGS=
    CXX=c++
    CXXFLAGS=
    cmd ./configure
fi

cmd make clean
cmd make -j4

cmd cd ".."
if test $(uname) = "Linux"; then
    cmd $CXX -lstdc++ -std=c++11 -fPIC -c -static -pthread ${CXXFLAGS} \
        -Isrc -I${CC_BUILD_DIR}/src src/rust_breakpad_linux.cc
    ar -M < link.mri
    mv librust_breakpad_client.a "$OUT_DIR"
    rm rust_breakpad_linux.o
fi
