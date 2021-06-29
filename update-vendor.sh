#!/bin/bash

set -e

rm -rf third-party/*
rm .cargo/config

cargo clean
cargo update $@

cargo vendor -- third-party > .cargo/config

# Unused for now.
rm -rf third-party/breakpad_sys/breakpad/
