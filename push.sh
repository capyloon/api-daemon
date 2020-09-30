#!/bin/bash
set -x -e

adb root && adb remount
adb shell stop api-daemon
echo "Stop api-daemon"

echo "Update the files of api-daemon"
adb push ./kaios-services-prefs.js /system/b2g/defaults/pref/
adb shell mkdir -p /data/local/service/api-daemon/http_root
adb shell rm -r /data/local/service/api-daemon/*
adb push ./prebuilts/armv7-linux-androideabi/api-daemon /data/local/service/api-daemon/api-daemon
adb push ./daemon/config-device-dev.toml /data/local/service/api-daemon/config.toml
adb push ./prebuilts/http_root /data/local/service/api-daemon/http_root

# used by hidl-utils.
adb push ./support/hidl-utils/libnative/libc++_shared.so /system/lib
adb push ./support/recovery/libnative/librecovery.so /system/lib

adb shell start api-daemon
echo "Start api-daemon"
