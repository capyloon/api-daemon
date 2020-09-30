#!/bin/bash

set -e -x

LD_PRELOAD=$HOME/dev/memory-profiler/target/release/libmemory_profiler.so ../target/debug/libsignal_sys-8653875d63c7c949
