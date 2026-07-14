#!/bin/bash
# Bundles the Android client (no server feature — talks to a remote server via SERVER_URL).
# --package-types apk forces a directly-installable .apk instead of dx's default .aab
# (Play Store bundle format), which CI can't sideload/verify as easily.
dx bundle --platform android --release --fullstack false --package-types apk --no-default-features --features mobile --out-dir bundle-android
