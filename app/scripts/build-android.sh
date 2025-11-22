#!/bin/bash

# Build and rename Android APK with date-based versioning
# Format: crosssave-cloud-YYYYMMDD-0.0.X.apk

set -e

# Get current date
DATE=$(date +%Y%m%d)

# Output directory
OUTPUT_DIR="$(dirname "$0")/../builds/android"
mkdir -p "$OUTPUT_DIR"

# Find existing builds for today
EXISTING_BUILDS=$(ls "$OUTPUT_DIR"/crosssave-cloud-$DATE-*.apk 2>/dev/null || true)

# Calculate next version number
if [ -z "$EXISTING_BUILDS" ]; then
    VERSION="0.0.1"
else
    # Extract the highest version number (macOS compatible)
    LAST_VERSION=$(echo "$EXISTING_BUILDS" | grep -o "0\.0\.[0-9]*" | grep -o "[0-9]*$" | sort -n | tail -1)
    NEXT_NUM=$((LAST_VERSION + 1))
    VERSION="0.0.$NEXT_NUM"
fi

echo "📋 Found existing builds: $(echo "$EXISTING_BUILDS" | wc -l | tr -d ' ')"
echo "🔢 Next version: $VERSION"
echo ""

# Build the APK
echo "Building Android APK..."
cd "$(dirname "$0")/.."

# Copy frontend build to Android assets
echo "Copying frontend assets..."
ANDROID_ASSETS_DIR="src-tauri/gen/android/app/src/main/assets"
mkdir -p "$ANDROID_ASSETS_DIR"

# Remove old frontend files (keep tauri.conf.json)
find "$ANDROID_ASSETS_DIR" -mindepth 1 ! -name "tauri.conf.json" -delete

# Copy frontend build
if [ -d "build" ]; then
    cp -r build/* "$ANDROID_ASSETS_DIR/"
    echo "✓ Frontend assets copied to Android"
else
    echo "⚠ Warning: build directory not found, running pnpm build first..."
    pnpm build
    cp -r build/* "$ANDROID_ASSETS_DIR/"
fi

pnpm tauri android build --target aarch64

# Source APK path
SOURCE_APK="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk"

# Sign the APK
echo "Signing APK..."
SIGNED_APK="src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk"
cp "$SOURCE_APK" "$SIGNED_APK"

export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
"$JAVA_HOME/bin/java" -jar ~/Library/Android/sdk/build-tools/35.0.0/lib/apksigner.jar sign \
    --ks ~/.android/debug.keystore \
    --ks-key-alias androiddebugkey \
    --ks-pass pass:android \
    --key-pass pass:android \
    "$SIGNED_APK"

# Final APK name
FINAL_NAME="crosssave-cloud-$DATE-$VERSION.apk"
FINAL_PATH="$OUTPUT_DIR/$FINAL_NAME"

# Copy to builds directory
cp "$SIGNED_APK" "$FINAL_PATH"

echo ""
echo "✅ Build complete!"
echo "📦 APK: $FINAL_PATH"
echo "📅 Date: $DATE"
echo "🔢 Version: $VERSION"
