#!/bin/bash
# Bundles the iOS Simulator client (no server feature — talks to a remote server via SERVER_URL).
# This produces an UNSIGNED raw .app bundle (--package-types ios, --codesign defaults to
# false) targeting the iOS Simulator only. It is NOT installable on a real iPhone —
# that requires a paid Apple Developer Program membership plus a signing certificate
# and provisioning profile, which this script deliberately does not attempt.
# Must run on macOS with Xcode installed (dx shells out to Xcode's toolchain).
#
# Usage: ./scripts/bundle_ios.sh [rustc-target-triple]
# Defaults to aarch64-apple-ios-sim, the Simulator target for Apple Silicon Macs
# (GitHub's macos-latest runners are Apple Silicon as of this writing).
# x86_64-apple-ios is the older Intel-Simulator target, for reference only.
TARGET="${1:-aarch64-apple-ios-sim}"
dx bundle --platform ios --release --fullstack false --package-types ios --target "$TARGET" --no-default-features --features mobile --out-dir bundle-ios
