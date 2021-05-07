#!/bin/bash
function pack_lib() {
    local src=$1
    local dst=$2
    mkdir -p ${dst}
    cp ${src}/dist/*.js ${dst}
    # libsignal has a non bundled file under ${src}/src that we need to copy.
    [ -d ${src}/js ] &&  cp ${src}/js/*.js ${dst}
    rm -rf ${dst}/*.gz
    # Install generated documentation if present
    [ -f ${src}/generated/*_service.html ] && cp ${src}/generated/*_service.html ${dst}/docs.html
    gzip --recursive --best --force ${dst}
}

function release_shared_lib() {
    local src=common/client
    local dst=$1/shared
    local build_type=$2

    pushd ${src} 2>/dev/null
    yarn install
    yarn ${build_type}session
    yarn ${build_type}core
    popd 2>/dev/null

    pack_lib common/client ${dst}
}

function release_service_lib() {
    local service=$1
    local src=${4:-services/${service}/client}
    local dst=$2/${service}
    local build_type=$3

    pushd ${src} 2>/dev/null
    yarn install
    yarn ${build_type}
    popd 2>/dev/null

    pack_lib ${src} ${dst}
}

function setup_xcompile_envs() {
    # This is the Rust target architecture, which may not directly map to the clang triple.
    # TODO: do the inverse mapping instead since we'll get the clang arch when running
    #       from Android.mk
    export TARGET_ARCH=${TARGET_ARCH:-armv7-linux-androideabi}
    export ANDROID_API=${ANDROID_API:-29}
    export ANDROID_PLATFORM=${ANDROID_PLATFORM:-android-29}
    LIB_SUFFIX=""

    case "$TARGET_ARCH" in
    armv7-linux-androideabi)
        TARGET_TRIPLE=armv7-linux-androideabi
	TARGET_INCLUDE=arm-linux-androideabi
        TOOLCHAIN_PREFIX=armv7a-linux-androideabi${ANDROID_API}
        ;;
    aarch64-linux-android)
        TARGET_TRIPLE=aarch64-linux-android
	TARGET_INCLUDE=${TARGET_TRIPLE}
        TOOLCHAIN_PREFIX=${TARGET_TRIPLE}${ANDROID_API}
        LIB_SUFFIX=64
        ;;
    x86_64-linux-android)
        TARGET_TRIPLE=x86_64-linux-android
	TARGET_INCLUDE=${TARGET_TRIPLE}
        TOOLCHAIN_PREFIX=${TARGET_TRIPLE}${ANDROID_API}
        LIB_SUFFIX=64
        ;;
    esac

    HOST_OS=$(uname -s)

    # Check that the BUILD_WITH_NDK_DIR environment variable is set
    # and build the .cargo/config file from it.
    if [ -n "${BUILD_WITH_NDK_DIR}" ]; then
	if [ ! -d "${BUILD_WITH_NDK_DIR}" ]; then
            echo "${BUILD_WITH_NDK_DIR} doesn't exixt."
	    exit 1
	fi
        export TOOLCHAIN_CC=${TOOLCHAIN_PREFIX}-clang
        export TOOLCHAIN_CXX=${TOOLCHAIN_PREFIX}-clang++
        export SYSROOT=${BUILD_WITH_NDK_DIR}/toolchains/llvm/prebuilt/linux-x86_64/sysroot
        export SYS_INCLUDE_DIR=${SYSROOT}/usr/include
        export ANDROID_NDK=${BUILD_WITH_NDK_DIR}
        export PATH=${ANDROID_NDK}/toolchains/llvm/prebuilt/linux-x86_64/bin:${PATH}

        echo "Building for ${TARGET_TRIPLE} using NDK '${BUILD_WITH_NDK_DIR}'"
    else
        echo "Set BUILD_WITH_NDK_DIR to your ndk directory to build"
        exit 2
    fi

    XCFLAGS="--sysroot=${SYSROOT} -I${SYS_INCLUDE_DIR} -I${SYS_INCLUDE_DIR}/${TARGET_INCLUDE}"

    export GIT_BUILD_INFO=$(
        git log -n 1 --pretty=format:"%H "
        date +%d/%m/%Y-%H:%M:%S
    )
}

function xcompile() {
    export CARGO_BUILD_TARGET=${TARGET_TRIPLE}
    export CARGO_CONFIG=$(pwd)/.cargo/config

    echo "Creating '$CARGO_CONFIG'"
    mkdir -p $(pwd)/.cargo
    cat <<EOF >$CARGO_CONFIG
[build]
target = "${TARGET_TRIPLE}"

[target.${TARGET_TRIPLE}]
linker = "${TOOLCHAIN_CC}"
rustflags = [
  "-C", "link-arg=--sysroot=${SYSROOT}",
  "-C", "link-arg=-L",
  "-C", "link-arg=${BUILD_WITH_NDK_DIR}/lib/gcc/${TARGET_TRIPLE}/4.9.x",
  "-C", "link-arg=-L",
  "-C", "link-arg=${BUILD_WITH_NDK_DIR}/sysroot/usr/lib/${TARGET_TRIPLE}/${ANDROID_API}",
  "-C", "link-arg=-Wl,-rpath,${GONK_DIR}/out/target/product/${GONK_PRODUCT}/system/lib${LIB_SUFFIX}",
  "-C", "opt-level=z",
]
EOF

    # unset the sysroot for the `backtrace` build deps so they don't pick up the wrong sysroot.
    unset CFLAGS

    # To add /usr/bin to $PATH, in order for host builds
    # of Rust crates to find 'cc' as a linker.
    # TODO: find a proper fix.
    export PATH=${PATH}:/usr/bin

    # Build native build-dependencies without overriding CC, CXX,... etc.
    if echo "$FEATURES" | grep "breakpad" > /dev/null 2>&1; then
        cargo build --verbose --target=${TARGET_TRIPLE} ${OPT} \
              -p native_build_deps
    fi

    export CC=${TOOLCHAIN_CC}
    export CXX=${TOOLCHAIN_CXX}
    export LD=${TOOLCHAIN_CC}

    # And set CFLAGS again for the remaining crates.
    export CFLAGS=${XCFLAGS}

    export TARGET_CC=${TOOLCHAIN_CC}
    export TARGET_LD=${TOOLCHAIN_CC}

    cat <<EOF >$(pwd)/env.txt
export CARGO_BUILD_TARGET=${TARGET_TRIPLE}
export CARGO_CONFIG=$(pwd)/.cargo/config
export CC=${TOOLCHAIN_CC}
export CXX=${TOOLCHAIN_CXX}
export LD=${TOOLCHAIN_CC}
export CFLAGS=${XCFLAGS}
export TARGET_CC=${TOOLCHAIN_CC}
export TARGET_LD=${TOOLCHAIN_CC}
EOF

    printenv
    rustc --version
    cargo --version
    cargo build --verbose --target=${TARGET_TRIPLE} ${FEATURES} ${OPT}
}

function generate_breakpad_symbols() {
    if [ "$TARGET_ARCH" == "armv7-linux-androideabi" ]; then
        generate_breakpad_symbols_armv7 $1
    fi
}

function generate_breakpad_symbols_armv7() {
    # Generate symbols
    HOST_OS=$(uname -s)
    if [ "$HOST_OS" == "Darwin" ]; then
        DUMP_SYMS=../tools/dump_syms/dump_syms_mac
        return
    else
        DUMP_SYMS=../tools/dump_syms/dump_syms
    fi
    echo python ../tools/dump_syms/generate_breakpad_symbols.py --dump-syms-dir ../tools/dump_syms \
        --symbols-dir ../target/${TARGET_TRIPLE}/${BUILD_TYPE}/symbols --binary $1
    python ../tools/dump_syms/generate_breakpad_symbols.py --dump-syms-dir ../tools/dump_syms \
        --symbols-dir ../target/${TARGET_TRIPLE}/${BUILD_TYPE}/symbols --binary $1
}

function xstrip() {
    echo "Stripping with `which llvm-strip`"
    # Explicitely strip the binary since even release builds have symbols.
    llvm-strip $1
}
