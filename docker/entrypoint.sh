#!/bin/sh
# Starts ts-gate in the background, then nginx in the foreground (PID 1).
set -eu

ts-gate &

nginx -t
exec nginx -g 'daemon off;'
