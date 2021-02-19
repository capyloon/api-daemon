#!/bin/bash

set -e

rm -rf third-party/*
rm .cargo/config

cargo clean
cargo update $@

cargo vendor -- third-party > .cargo/config

# Unused for now.
rm -rf third-party/breakpad_sys/breakpad/

# Remove useless windows libraries
rm -rf third-party/winapi-x86_64-pc-windows-gnu/lib
rm -rf third-party/winapi-i686-pc-windows-gnu/lib