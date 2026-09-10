#!/bin/bash
# Runs the Android native client and points it at dev_web.sh's server.
#
# `adb reverse` tunnels the device's 127.0.0.1:8080 to the dev machine's, for emulator and USB
# device alike. Unlike the 10.0.2.2 alias it also satisfies Android's WebView cleartext policy,
# which only allows plain http to 127.0.0.1 — <img> tags go through the WebView while server-fn
# and websocket calls use reqwest, so without it images silently fail while gameplay works.
#
# Usage:
#   ./scripts/dev_android.sh                                   # emulator or real device over adb
#   SERVER_URL=http://<lan-ip>:8080 ./scripts/dev_android.sh   # real device without adb reverse (e.g. wifi-adb on a different subnet)

# `adb` isn't always on PATH (e.g. VS Code's git-bash task shell on Windows) even when
# ANDROID_HOME/ANDROID_SDK_ROOT is set, so fall back to locating it under the SDK root.
if command -v adb >/dev/null 2>&1; then
    ADB=adb
elif [ -n "$ANDROID_HOME" ] && [ -x "$ANDROID_HOME/platform-tools/adb.exe" ]; then
    ADB="$ANDROID_HOME/platform-tools/adb.exe"
elif [ -n "$ANDROID_HOME" ] && [ -x "$ANDROID_HOME/platform-tools/adb" ]; then
    ADB="$ANDROID_HOME/platform-tools/adb"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -x "$ANDROID_SDK_ROOT/platform-tools/adb.exe" ]; then
    ADB="$ANDROID_SDK_ROOT/platform-tools/adb.exe"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -x "$ANDROID_SDK_ROOT/platform-tools/adb" ]; then
    ADB="$ANDROID_SDK_ROOT/platform-tools/adb"
else
    ADB=""
fi

if [ -n "$ADB" ]; then
    "$ADB" reverse tcp:8080 tcp:8080 || echo "warning: adb reverse failed — is a device/emulator connected? ($ADB devices)"
else
    echo "warning: adb not found (checked PATH, \$ANDROID_HOME/platform-tools, \$ANDROID_SDK_ROOT/platform-tools) — skipping adb reverse. Set ANDROID_HOME or add platform-tools to PATH, or images/reconnect may fail."
fi
SERVER_URL="${SERVER_URL:-http://127.0.0.1:8080}" dx serve --platform android --no-default-features --features mobile
