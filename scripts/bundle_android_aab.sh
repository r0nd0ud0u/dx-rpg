#!/bin/bash
# Bundles the Android App Bundle (.aab) for Play Store submission (no server
# feature — talks to a remote server via SERVER_URL), signed with the release
# keystore configured in [bundle.android] at build time (see the CI workflow's
# "Patch Dioxus.toml with release signing config" step, which points those
# fields at a keystore decoded from a GitHub secret — never the committed
# android/debug.keystore used by bundle_mobile.sh for sideloadable test APKs).
#
# Usage: ./scripts/bundle_android_aab.sh [rustc-target-triple]
# Defaults to aarch64-linux-android (arm64-v8a); see bundle_mobile.sh for why
# 32-bit targets aren't supported by dioxus 0.7.9's manganis crate. Play
# accepts an arm64-v8a-only bundle fine — 32-bit device support isn't
# mandatory, only a 64-bit build is required when native code is included.
TARGET="${1:-aarch64-linux-android}"
dx bundle --platform android --release --fullstack false --package-types aab --target "$TARGET" --no-default-features --features mobile --out-dir bundle-android-aab
