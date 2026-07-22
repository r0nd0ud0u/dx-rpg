#!/bin/bash
# Runs the Android native client and points it at dev_web.sh's server.
# Usage:
#   ./scripts/dev_android.sh                                   # standard Android emulator (10.0.2.2 -> host loopback)
#   SERVER_URL=http://<lan-ip>:8080 ./scripts/dev_android.sh   # real physical device on the same LAN
SERVER_URL="${SERVER_URL:-http://10.0.2.2:8080}" dx serve --platform android --no-default-features --features mobile
