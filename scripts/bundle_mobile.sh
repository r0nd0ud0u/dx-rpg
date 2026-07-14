#!/bin/bash
# Bundles the Android client (no server feature — talks to a remote server via SERVER_URL).
dx bundle --platform android --release --fullstack false --no-default-features --features mobile --out-dir bundle-android
