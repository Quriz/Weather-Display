#!/bin/sh

while true; do
    /usr/local/bin/app
    echo "App crashed with exit code $?. Restarting..." >&2
    sleep 10
done
