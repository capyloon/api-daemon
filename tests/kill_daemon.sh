#!/bin/bash

set -e

echo "Trying to kill api-daemon"

# Kill the daemon
pid=$(ps -ef | grep api-daemon | grep -v grep | awk '{print $2}');
if [ -n "$pid" ]; then
    kill -9 $pid;
    echo "Killed api-daemon (pid $pid)";
    exit 0;
fi

exit 1;