#!/bin/bash
# Bundles the desktop client (no server feature — talks to a remote server via SERVER_URL).
package_type_args=()
if [ "$(uname)" = "Linux" ]; then
  # Also emit .deb/.rpm alongside the default AppImage: unlike AppImage, they can
  # declare libwebkit2gtk-4.1-0 / webkit2gtk4.1 as a package dependency (see
  # Dioxus.toml [bundle.deb]/[bundle.rpm]), so apt/dnf install it automatically.
  package_type_args=(--package-types appimage --package-types deb --package-types rpm)
fi
dx bundle --platform desktop --release --fullstack false --no-default-features --features desktop --out-dir bundle-desktop "${package_type_args[@]}"
