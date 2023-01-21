#!/bin/bash

set -e

rm -rf third-party/*
rm .cargo/config

cargo clean
cargo update $@

cargo vendor -- third-party > .cargo/config

# Unused for now.
rm -rf third-party/breakpad_sys/breakpad/

# Unused windows libraries
rm -rf third-party/winapi-x86_64-pc-windows-gnu/lib
rm -rf third-party/winapi-i686-pc-windows-gnu/lib
rm -rf third-party/windows_i686_gnu/lib
rm -rf third-party/windows_x86_64_gnu/lib
rm -rf third-party/windows-sys/src/Windows/
rm -rf third-party/windows_x86_64_gnu-0.36.1/lib
rm -rf third-party/windows_i686_gnu-0.36.1/lib
rm -rf third-party/windows-sys-0.36.1/src/Windows/
