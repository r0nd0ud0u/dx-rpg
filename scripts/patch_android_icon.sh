#!/bin/bash
# dioxus-cli 0.7.9's Android bundler hardcodes its launcher icon
# (packages/cli/src/build/android.rs writes include_bytes!'d mipmap/
# drawable templates unconditionally — no Dioxus.toml field reaches
# this code path at all). What's baked in isn't even a Dioxus-branded
# placeholder: it's Android Studio's stock "New Project" default icon
# (green grid background, white robot silhouette foreground) — verified
# by downloading a real release APK and extracting res/*.webp directly.
#
# There's no config-level fix, so this surgically patches the built
# APK's icon resources in place instead:
#  1. Resize assets/icon-512.png to the 5 legacy mipmap densities and
#     overwrite those exact files (plain raster swap, no compiler
#     needed — Android resolves these by content, not by their
#     resource-shrinker-obfuscated on-disk names). These only ever get
#     used on API < 26; every modern device takes the adaptive-icon path
#     in step 2 instead, so a correct-looking bitmap here proves nothing
#     about what a real phone will show.
#  2. Recompile the adaptive-icon background/foreground drawables via
#     aapt2 from android/ic_launcher_{background,foreground}.xml. This
#     can't be a raw byte swap like the mipmaps: both are compiled binary
#     XML (Android vector-drawable format), so producing valid
#     replacements requires a real compiler. Both are self-contained
#     vectors — see the comment in ic_launcher_foreground.xml for why the
#     foreground must NOT be a <bitmap> pointing at @mipmap/ic_launcher
#     (it resolves back to this very adaptive icon on API 26+, and the
#     resulting self-reference makes the launcher fall back to the
#     generic default Android app icon).
#  3. mipmap-anydpi-v26/ic_launcher.xml (the adaptive-icon wrapper) is
#     left untouched — it already references the background/foreground
#     drawables above by resource ID, and those IDs don't change just
#     because the drawables' content does.
#  4. Splice the new bytes into the existing APK zip by exact path,
#     zipalign, and re-sign with the same debug keystore used to build
#     it — modifying contents invalidates the original signature, so
#     this must produce a byte-identical certificate or installs over
#     a previous build will fail with "package conflicts with an
#     existing package" (see bundle_to_asset.yml's "Print APK signing
#     certificate fingerprint" step, which would have caught a mismatch
#     here anyway, but re-signing correctly means it won't).
#
# Every resource path/ID below is read out of the APK being patched via
# `aapt2 dump resources` rather than hardcoded: dx's exact resource-ID
# assignment and the resource shrinker's obfuscated on-disk file names
# ("res/BJ.xml") aren't documented guarantees, so this adapts to whatever
# the build actually produced and hard-fails if the layout ever changes.
#
# Requires: ANDROID_HOME (build-tools 34.0.0 + platforms;android-34),
# imagemagick, run from the repo root with bundle-android/*.apk already
# built (see bundle_mobile.sh).
set -euo pipefail

aapt2=$(find "$ANDROID_HOME/build-tools" -name aapt2 -type f | sort -V | tail -n1)
zipalign=$(find "$ANDROID_HOME/build-tools" -name zipalign -type f | sort -V | tail -n1)
apksigner=$(find "$ANDROID_HOME/build-tools" -name apksigner -type f | sort -V | tail -n1)
android_jar="$ANDROID_HOME/platforms/android-34/android.jar"
# -print -quit rather than `| head -n1`: under `set -o pipefail`, head exiting
# after the first line can SIGPIPE find and fail the whole script.
apk=$(find bundle-android -name '*.apk' -print -quit)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

dump=$("$aapt2" dump resources "$apk")
bg_path=$(echo "$dump" | grep -A1 ' drawable/ic_launcher_background$' | grep -oP 'res/\S+\.xml')
fg_path=$(echo "$dump" | grep -A1 ' drawable/ic_launcher_foreground$' | grep -oP 'res/\S+\.xml')
mdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(mdpi)' | grep -oP 'res/\S+\.webp')
hdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(hdpi)' | grep -oP 'res/\S+\.webp')
xhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xhdpi)' | grep -oP 'res/\S+\.webp')
xxhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xxhdpi)' | grep -oP 'res/\S+\.webp')
xxxhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xxxhdpi)' | grep -oP 'res/\S+\.webp')
for v in bg_path fg_path mdpi_path hdpi_path xhdpi_path xxhdpi_path xxxhdpi_path; do
  if [ -z "${!v}" ]; then
    echo "::error::Failed to extract $v from aapt2 dump — dx-cli's Android icon resource layout may have changed, this patch step needs updating." >&2
    exit 1
  fi
done

# Ubuntu's `imagemagick` apt package is still ImageMagick 6 (noble ships
# 6.9.12), which has no `magick` binary at all — only `convert`. `magick` is
# an IM7-only unified entrypoint. `convert` still works fine on IM7 too (as a
# deprecated compat alias), so this stays portable across both.
mkdir -p "$work/densities"
convert assets/icon-512.png -resize 48x48   "$work/densities/mdpi.webp"
convert assets/icon-512.png -resize 72x72   "$work/densities/hdpi.webp"
convert assets/icon-512.png -resize 96x96   "$work/densities/xhdpi.webp"
convert assets/icon-512.png -resize 144x144 "$work/densities/xxhdpi.webp"
convert assets/icon-512.png -resize 192x192 "$work/densities/xxxhdpi.webp"

# Compile the two vector drawables. This links a throwaway package purely to
# get aapt2 to emit compiled binary XML; only the two drawable outputs are
# taken from it, and neither references any other resource, so nothing here
# depends on the real APK's resource IDs.
proj="$work/proj"
mkdir -p "$proj/res/drawable"
cat > "$proj/AndroidManifest.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="io.github.r0ndoudou.rpgadventure.iconpatch">
    <application android:label="iconpatch"/>
</manifest>
EOF
cp android/ic_launcher_background.xml "$proj/res/drawable/ic_launcher_background.xml"
cp android/ic_launcher_foreground.xml "$proj/res/drawable/ic_launcher_foreground.xml"

"$aapt2" compile --dir "$proj/res" -o "$proj/compiled.zip"
"$aapt2" link -o "$proj/linked.apk" -I "$android_jar" --manifest "$proj/AndroidManifest.xml" \
  --no-version-vectors --no-auto-version --min-sdk-version 24 \
  "$proj/compiled.zip"

mkdir -p "$work/compiled_out"
unzip -o -q "$proj/linked.apk" \
  "res/drawable/ic_launcher_background.xml" \
  "res/drawable/ic_launcher_foreground.xml" \
  -d "$work/compiled_out"

staging="$work/staging"
mkdir -p "$staging/$(dirname "$mdpi_path")"
cp "$work/densities/mdpi.webp" "$staging/$mdpi_path"
cp "$work/densities/hdpi.webp" "$staging/$hdpi_path"
cp "$work/densities/xhdpi.webp" "$staging/$xhdpi_path"
cp "$work/densities/xxhdpi.webp" "$staging/$xxhdpi_path"
cp "$work/densities/xxxhdpi.webp" "$staging/$xxxhdpi_path"
cp "$work/compiled_out/res/drawable/ic_launcher_background.xml" "$staging/$bg_path"
cp "$work/compiled_out/res/drawable/ic_launcher_foreground.xml" "$staging/$fg_path"

patched="$work/patched.apk"
cp "$apk" "$patched"
( cd "$staging" && zip -q "$patched" "$mdpi_path" "$hdpi_path" "$xhdpi_path" "$xxhdpi_path" "$xxxhdpi_path" "$bg_path" "$fg_path" )

"$zipalign" -p -f 4 "$patched" "$work/aligned.apk"
# v4 signing off: it emits a separate <name>.apk.idsig sidecar next to the
# output, which dx's own build never produced and which would just be litter
# in bundle-android/. v2/v3 (enabled by default) are what sideloaded installs
# actually verify.
"$apksigner" sign --ks android/debug.keystore --ks-pass pass:android --key-pass pass:android \
  --ks-key-alias androiddebugkey --v4-signing-enabled false \
  --out "$apk.icon-patched" "$work/aligned.apk"
"$apksigner" verify "$apk.icon-patched"
mv "$apk.icon-patched" "$apk"

# Guard against silently shipping a broken icon again: assert the patched
# foreground really is a self-contained <vector> and not a <bitmap> pointing
# back at the adaptive icon (the exact defect that made earlier builds fall
# back to the stock Android icon on every API 26+ device).
# Captured first, then matched with awk over a here-string: piping aapt2
# straight into `grep -m1` makes grep exit early, SIGPIPEs aapt2, and under
# `set -o pipefail` that kills the script even though the patch succeeded.
fg_dump=$("$aapt2" dump xmltree --file "$fg_path" "$apk")
fg_root=$(awk 'match($0, /E: [A-Za-z0-9_-]+/) { print substr($0, RSTART + 3, RLENGTH - 3); exit }' <<< "$fg_dump")
if [ "$fg_root" != "vector" ]; then
  echo "::error::Patched adaptive-icon foreground has root element <$fg_root>, expected <vector>." >&2
  exit 1
fi
echo "Patched launcher icon OK (foreground root element: <$fg_root>)"
