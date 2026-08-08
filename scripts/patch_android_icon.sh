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
#     resource-shrinker-obfuscated on-disk names).
#  2. Recompile just the adaptive-icon background/foreground drawables
#     via aapt2 — background becomes a solid vector matching our navy
#     brand color, foreground becomes a <bitmap> pointing at the same
#     mipmap/ic_launcher entry from step 1. This can't be a raw byte
#     swap like the mipmaps: these two are themselves compiled binary
#     XML (Android vector-drawable format), so producing valid
#     replacements requires a real compiler. `--stable-ids` pins the
#     foreground's @mipmap/ic_launcher reference to mipmap/ic_launcher's
#     *actual* resource ID in the real APK (extracted via `aapt2 dump
#     resources`, never hardcoded — dx's exact ID assignment isn't a
#     documented guarantee, so this reads whatever this build actually
#     produced instead of assuming it matches a previous run).
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
# Requires: ANDROID_HOME (build-tools 34.0.0 + platforms;android-34),
# imagemagick, run from the repo root with bundle-android/*.apk already
# built (see bundle_mobile.sh).
set -euo pipefail

aapt2=$(find "$ANDROID_HOME/build-tools" -name aapt2 -type f | sort -V | tail -n1)
zipalign=$(find "$ANDROID_HOME/build-tools" -name zipalign -type f | sort -V | tail -n1)
apksigner=$(find "$ANDROID_HOME/build-tools" -name apksigner -type f | sort -V | tail -n1)
android_jar="$ANDROID_HOME/platforms/android-34/android.jar"
apk=$(find bundle-android -name '*.apk' | head -n1)
work=$(mktemp -d)

dump=$("$aapt2" dump resources "$apk")
mipmap_id=$(echo "$dump" | grep -oP '0x[0-9a-f]+(?= mipmap/ic_launcher$)')
bg_path=$(echo "$dump" | grep -A1 ' drawable/ic_launcher_background$' | grep -oP 'res/\S+\.xml')
fg_path=$(echo "$dump" | grep -A1 ' drawable/ic_launcher_foreground$' | grep -oP 'res/\S+\.xml')
mdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(mdpi)' | grep -oP 'res/\S+\.webp')
hdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(hdpi)' | grep -oP 'res/\S+\.webp')
xhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xhdpi)' | grep -oP 'res/\S+\.webp')
xxhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xxhdpi)' | grep -oP 'res/\S+\.webp')
xxxhdpi_path=$(echo "$dump" | grep -A6 ' mipmap/ic_launcher$' | grep '(xxxhdpi)' | grep -oP 'res/\S+\.webp')
for v in mipmap_id bg_path fg_path mdpi_path hdpi_path xhdpi_path xxhdpi_path xxxhdpi_path; do
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

proj="$work/proj"
mkdir -p "$proj/res/drawable" "$proj/res/drawable-v24" "$proj/res/mipmap-mdpi"
cat > "$proj/AndroidManifest.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="io.github.r0ndoudou.rpgadventure.iconpatch">
    <application android:label="iconpatch"/>
</manifest>
EOF
cat > "$proj/res/drawable/ic_launcher_background.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp" android:height="108dp"
    android:viewportWidth="108" android:viewportHeight="108">
    <path android:fillColor="#080c14" android:pathData="M0,0h108v108h-108z" />
</vector>
EOF
cat > "$proj/res/drawable-v24/ic_launcher_foreground.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<bitmap xmlns:android="http://schemas.android.com/apk/res/android"
    android:src="@mipmap/ic_launcher"
    android:gravity="center" />
EOF
echo "io.github.r0ndoudou.rpgadventure.iconpatch:mipmap/ic_launcher = $mipmap_id" > "$proj/stable_ids.txt"
# Placeholder so @mipmap/ic_launcher resolves during this link; only the
# background/foreground XML outputs below are actually used afterward.
cp "$work/densities/mdpi.webp" "$proj/res/mipmap-mdpi/ic_launcher.webp"

"$aapt2" compile --dir "$proj/res" -o "$proj/compiled.zip"
"$aapt2" link -o "$proj/linked.apk" -I "$android_jar" --manifest "$proj/AndroidManifest.xml" \
  --no-version-vectors --no-auto-version --min-sdk-version 24 \
  --stable-ids "$proj/stable_ids.txt" --package-id 0x7f \
  "$proj/compiled.zip"

mkdir -p "$work/compiled_out"
unzip -o -q "$proj/linked.apk" "res/drawable/ic_launcher_background.xml" "res/drawable-v24/ic_launcher_foreground.xml" -d "$work/compiled_out"

staging="$work/staging"
mkdir -p "$staging/$(dirname "$mdpi_path")"
cp "$work/densities/mdpi.webp" "$staging/$mdpi_path"
cp "$work/densities/hdpi.webp" "$staging/$hdpi_path"
cp "$work/densities/xhdpi.webp" "$staging/$xhdpi_path"
cp "$work/densities/xxhdpi.webp" "$staging/$xxhdpi_path"
cp "$work/densities/xxxhdpi.webp" "$staging/$xxxhdpi_path"
cp "$work/compiled_out/res/drawable/ic_launcher_background.xml" "$staging/$bg_path"
cp "$work/compiled_out/res/drawable-v24/ic_launcher_foreground.xml" "$staging/$fg_path"

patched="$work/patched.apk"
cp "$apk" "$patched"
( cd "$staging" && zip -q "$patched" "$mdpi_path" "$hdpi_path" "$xhdpi_path" "$xxhdpi_path" "$xxxhdpi_path" "$bg_path" "$fg_path" )

"$zipalign" -p -f 4 "$patched" "$work/aligned.apk"
"$apksigner" sign --ks android/debug.keystore --ks-pass pass:android --key-pass pass:android \
  --ks-key-alias androiddebugkey --out "$apk.icon-patched" "$work/aligned.apk"
"$apksigner" verify "$apk.icon-patched"
mv "$apk.icon-patched" "$apk"
rm -rf "$work"
