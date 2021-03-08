#!/bin/bash
set -e

if [ -z "${C_LIBRARY_DIR}" ] || [ -z "${CC_BUILD_DIR}" ];
then
    echo "This script is supposed to be run by \`build.rs\`!" >&2
    exit 1
fi

if ! [ "${TARGET}" = "armv7-linux-androideabi" ];
then
    CC=cc
    CFLAGS=
    CXX=c++
    CXXFLAGS=
fi

cmd() {
    echo " • Running: $* …" >&2
    "$@"
}

configure() {
    if [ "${TARGET}" = "armv7-linux-androideabi" ];
    then
        #patch for fix compile conflict in wchar.h
        cmd rm -f src/common/android/testing/include/wchar.h

        cmd ./configure --host="${TARGET}" \
                    --disable-processor \
                    --disable-tools
    else
        cmd ./configure
    fi
}

if test $(uname) = "Linux"; then
    if ! [ -e "${CC_BUILD_DIR}/.git" ];
    then
        cmd git clone https://chromium.googlesource.com/breakpad/breakpad ./${CC_BUILD_DIR} 
        cmd git clone https://chromium.googlesource.com/linux-syscall-support ./${CC_BUILD_DIR}/src/third_party/lss

        cmd pushd "${CC_BUILD_DIR}"
        configure
        cmd popd
    fi

    if ! [ -d "src/generated" ];
    then 
        cmd mkdir src/generated
    fi
fi

cmd pushd "${CC_BUILD_DIR}"
# Update breakpad.
cmd git pull

# If any file changed, clean the build and re-configure
if ! [[ $(git diff-tree -r --name-only ORIG_HEAD HEAD) = "" ]];
then
    cmd make clean
    configure
fi

cmd make -j4

cmd popd
if test $(uname) = "Linux"; then
    cmd $CXX -lstdc++ -std=c++11 -fPIC -c -static -pthread ${CXXFLAGS} \
        -Isrc -I${CC_BUILD_DIR}/src src/rust_breakpad_linux.cc
    ar -M < link.mri
    mv librust_breakpad_client.a "$OUT_DIR"
    rm rust_breakpad_linux.o
fi
