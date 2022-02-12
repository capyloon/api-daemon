#!/bin/bash
if [ -z "${C_LIBRARY_DIR}" ] || [ -z "${C_BUILD_DIR}" ];
then
	echo "This script is supposed to be run by \`build.rs\`!" >&2
	exit 1
fi

cmd() {
	echo " • Running: $* …" >&2
	"$@"
}

set -e

if [ "${TARGET}" = "x86_64-unknown-linux-gnu" ];
then
       CC=cc
       XCFLAGS=
else
       source ../../utils.sh
       setup_xcompile_envs
       
       echo "Linker is `which aarch64-apple-darwin-ld`"
       EXTRA_FLAGS="-fuse-ld=${OSX_CROSS}/cctools/bin/aarch64-apple-darwin-ld -L${OSX_CROSS}/MacOSX11.0.sdk/usr/lib"
fi

# Download and enter C library directory
if ! [ -e "${C_LIBRARY_DIR}/.git" ];
then
	cmd git submodule update --init
fi
cd "${C_LIBRARY_DIR}"

# Store full library checkout path
cmake_library_dir="$(pwd)"

# Switch to Cargo-provided build directory
mkdir -p "${C_BUILD_DIR}"
cd "${C_BUILD_DIR}"

# Build C library using CMake
if [ "${DEBUG}" = "true" ];
then
	cmake_build_type="Debug"
else
	cmake_build_type="Release"
fi

# When building in the aosp build system, use the prebuilt cmake
# that is configured properly.
if [ -f ${GONK_DIR}/prebuilts/cmake/linux-x86/bin/cmake ];
then
  cmake_prog="${GONK_DIR}/prebuilts/cmake/linux-x86/bin/cmake"
else
  cmake_prog="cmake"
fi

cmd ${cmake_prog} -DCMAKE_BUILD_TYPE="${cmake_build_type}" \
          -DCMAKE_OSX_SYSROOT=/media/external/dev/osx-cross/MacOSX11.0.sdk \
          -DCMAKE_C_COMPILER="${CC}" \
          -DCMAKE_C_FLAGS="-fPIC ${EXTRA_FLAGS} -O${OPT_LEVEL} ${XCFLAGS}" "${cmake_library_dir}"
cmd make -j"${NUM_JOBS}"

