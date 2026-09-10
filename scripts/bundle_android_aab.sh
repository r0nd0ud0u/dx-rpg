#!/bin/bash
# Android App Bundle (.aab) for the Play Store. Signed with the release keystore CI patches
# into [bundle.android] from a GitHub secret — never the committed debug keystore.
#
# Usage: ./scripts/bundle_android_aab.sh [rustc-target-triple]
# Defaults to aarch64-linux-android; 32-bit isn't supported (see bundle_mobile.sh) and Play
# only requires a 64-bit build.
TARGET="${1:-aarch64-linux-android}"
dx bundle --platform android --release --fullstack false --package-types aab --target "$TARGET" --no-default-features --features mobile --out-dir bundle-android-aab
