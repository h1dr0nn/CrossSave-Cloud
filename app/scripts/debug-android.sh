#!/bin/bash

# View Android logcat filtered for CrossSave Cloud app crashes
# This helps debug runtime crashes on real devices

echo "📱 Monitoring Android logcat for CrossSave Cloud..."
echo "🔍 Filtering for crashes and errors..."
echo ""
echo "Press Ctrl+C to stop monitoring"
echo "----------------------------------------"

# Add Android SDK platform-tools to PATH
export PATH="$PATH:$HOME/Library/Android/sdk/platform-tools"

# Check if adb is available
if ! command -v adb &> /dev/null; then
    echo "❌ Error: adb not found!"
    echo "Please make sure Android SDK platform-tools is installed."
    exit 1
fi

adb logcat -c  # Clear previous logs
adb logcat | grep -E "(AndroidRuntime|FATAL|crosssave|com.h1dr0n.crosssave_cloud|chromium|Console|RustWebView|RustStdoutStderr)"
