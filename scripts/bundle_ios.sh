#!/bin/bash
# iOS Simulator client: an UNSIGNED .app, Simulator only. A real iPhone needs a paid Apple
# Developer membership plus a certificate and provisioning profile. macOS + Xcode required.
#
# Usage: ./scripts/bundle_ios.sh [rustc-target-triple]
# Defaults to aarch64-apple-ios-sim (Apple Silicon); x86_64-apple-ios is the Intel one.
TARGET="${1:-aarch64-apple-ios-sim}"
dx bundle --platform ios --release --fullstack false --package-types ios --target "$TARGET" --no-default-features --features mobile --out-dir bundle-ios
