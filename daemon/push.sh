#!/bin/bash
set -x -e
adb remount
adb push ../target/armv7-linux-androideabi/debug/api-daemon /system/kaios/api-daemon
adb shell chmod 777 /system/kaios/api-daemon
adb push config-device-dev.toml /system/kaios/config.toml
