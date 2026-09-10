#!/bin/bash
# dioxus-cli 0.7.9 hardcodes the Android launcher icon (Android Studio's stock
# "New Project" default); no Dioxus.toml field reaches that code path, so patch the
# built APK instead:
#  1. Raster-swap the 5 legacy mipmap densities from assets/icon-512.png (API < 26 only).
#  2. Recompile the adaptive-icon drawables with aapt2 from android/ic_launcher_*.xml —
#     these are compiled binary XML, so a byte swap won't do. The foreground must stay a
#     self-contained vector: a <bitmap> pointing at @mipmap/ic_launcher self-references and
#     the launcher falls back to the generic Android icon.
#  3. Re-zip, zipalign and re-sign with the same debug keystore — patching invalidates the
#     original signature, and a different cert breaks installs over a previous build.
#
# Resource paths/IDs come from `aapt2 dump resources`, never hardcoded: the shrinker
# obfuscates on-disk names ("res/BJ.xml"). Hard-fails if the layout changes.
#
# Requires: ANDROID_HOME (build-tools 34.0.0 + platforms;android-34), imagemagick, run from
# the repo root with bundle-android/*.apk already built (see bundle_mobile.sh).
set -euo pipefail

aapt2=$(find "$ANDROID_HOME/build-tools" -name aapt2 -type f | sort -V | tail -n1)
zipalign=$(find "$ANDROID_HOME/build-tools" -name zipalign -type f | sort -V | tail -n1)
apksigner=$(find "$ANDROID_HOME/build-tools" -name apksigner -type f | sort -V | tail -n1)
android_jar="$ANDROID_HOME/platforms/android-34/android.jar"
# -print -quit, not `| head -n1`: head exiting early SIGPIPEs find under pipefail.
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

# `convert`, not `magick`: Ubuntu still ships ImageMagick 6 (no `magick` binary);
# `convert` works on both 6 and 7.
mkdir -p "$work/densities"
convert assets/icon-512.png -resize 48x48   "$work/densities/mdpi.webp"
convert assets/icon-512.png -resize 72x72   "$work/densities/hdpi.webp"
convert assets/icon-512.png -resize 96x96   "$work/densities/xhdpi.webp"
convert assets/icon-512.png -resize 144x144 "$work/densities/xxhdpi.webp"
convert assets/icon-512.png -resize 192x192 "$work/densities/xxxhdpi.webp"

# Compile the two vector drawables. The throwaway package just makes aapt2 emit
# compiled binary XML; neither drawable references other resources, so no ID coupling.
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
# v4 off: it emits a .apk.idsig sidecar dx never produced. v2/v3 are what sideloads verify.
"$apksigner" sign --ks android/debug.keystore --ks-pass pass:android --key-pass pass:android \
  --ks-key-alias androiddebugkey --v4-signing-enabled false \
  --out "$apk.icon-patched" "$work/aligned.apk"
"$apksigner" verify "$apk.icon-patched"
mv "$apk.icon-patched" "$apk"

# Assert the foreground is a self-contained <vector>, not a <bitmap> pointing back at
# the adaptive icon — that self-reference is what fell back to the stock icon on API 26+.
# Captured then matched with awk: `grep -m1` would SIGPIPE aapt2 under pipefail.
fg_dump=$("$aapt2" dump xmltree --file "$fg_path" "$apk")
fg_root=$(awk 'match($0, /E: [A-Za-z0-9_-]+/) { print substr($0, RSTART + 3, RLENGTH - 3); exit }' <<< "$fg_dump")
if [ "$fg_root" != "vector" ]; then
  echo "::error::Patched adaptive-icon foreground has root element <$fg_root>, expected <vector>." >&2
  exit 1
fi
echo "Patched launcher icon OK (foreground root element: <$fg_root>)"
