#!/bin/bash

set -e

rm -rf third-party/*
rm -f .cargo/config

cargo clean
cargo update $@

cargo vendor -- third-party > .cargo/config

# Unused for now.
rm -rf third-party/breakpad_sys/breakpad/

# Unused windows libraries
rm -rf third-party/winapi-x86_64*/lib
rm -rf third-party/winapi-i686*/lib
rm -rf third-party/windows_i686*/lib
rm -rf third-party/windows_x86_64*/lib
rm -rf third-party/windows_aarch64*/lib
rm -rf third-party/windows-*/src/Windows/
rm -rf third-party/windows/src/Windows/

du -h --max-depth=0 third-party/
