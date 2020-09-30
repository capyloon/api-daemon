#!/bin/bash

set -e

# Copy file/dir to device via ADB
#
# Usage: adb_push <local> <remote>
# The behavior would act like "adb push <local>/* <remote>"
adb_push()
{
  if [ -d $1 ]; then
    LOCAL_FILES="$1/*"
    for file in $LOCAL_FILES
    do
      adb_push $file "$2/$(basename $file)"
    done
  else
    printf "push: %-30s " $file
    adb push $1 $2
  fi
}

adb wait-for-device
adb root
adb wait-for-device
adb remount

adb shell stop b2g
# adb shell rm -r /data/local/webapps
# adb_push webapps /data/local/webapps
adb shell rm -rf /system/b2g/webapps/*
adb shell mkdir -p /system/b2g/webapps
adb_push webapps /system/b2g/webapps
adb shell start b2g
