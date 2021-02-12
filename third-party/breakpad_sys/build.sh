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

    cmd ./configure --host="${TARGET}" \
                --disable-processor \
                --disable-tools
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
