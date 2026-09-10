#!/bin/bash
# Bundles the Android client (no server feature — talks to a remote server via SERVER_URL).
# --package-types apk forces a directly-installable .apk instead of dx's default .aab
# (Play Store bundle format), which CI can't sideload/verify as easily.
#
# Usage: ./scripts/bundle_mobile.sh [rustc-target-triple]
# The triple must be explicit: dx's own default may not match the device, giving an APK that
# fails to install ("app not compatible with this device"). Defaults to aarch64-linux-android;
# x86_64-linux-android works for emulators. 32-bit targets don't — dioxus 0.7.9's manganis
# errors with "Only 64-bit Android targets are supported".
TARGET="${1:-aarch64-linux-android}"
dx bundle --platform android --release --fullstack false --package-types apk --target "$TARGET" --no-default-features --features mobile --out-dir bundle-android
