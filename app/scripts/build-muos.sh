#!/bin/bash

# Build CrossSave Cloud for MUOS
# This script packages the Linux binaries into a MUOS-compatible structure
# Output: Single .muxapp file in builds/linux/MUOS/
# Structure inside .muxapp:
#   CrossSave Cloud/ (Single root folder)
#     mux_launch.sh (Launcher inside app folder)
#     bin/
#     assets/
#     ...

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎮 CrossSave Cloud - MUOS Builder${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Ensure we are in the project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

# 1. Ensure Linux binaries exist
if [ ! -d "builds/linux/x86_64" ] || [ ! -d "builds/linux/aarch64" ]; then
    echo -e "${YELLOW}⚠️  Linux builds not found. Running Docker build first...${NC}"
    ./scripts/build-linux-docker.sh
fi

# Find latest binaries
BIN_X86=$(find builds/linux/x86_64 -name "crosssave-cloud-*-x86_64" | sort | tail -1)
BIN_ARM=$(find builds/linux/aarch64 -name "crosssave-cloud-*-aarch64" | sort | tail -1)

if [ -z "$BIN_X86" ] || [ -z "$BIN_ARM" ]; then
    echo -e "${RED}❌ Error: Could not find built binaries.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Found binaries:${NC}"
echo "  x86: $(basename "$BIN_X86")"
echo "  ARM: $(basename "$BIN_ARM")"
echo ""

# 2. Setup Directories
OUTPUT_DIR="$PROJECT_ROOT/builds/linux/MUOS"
mkdir -p "$OUTPUT_DIR"

# Create a temporary working directory
WORK_DIR=$(mktemp -d)
APP_NAME="CrossSave Cloud"  # Space allowed per checklist
APP_DIR="$WORK_DIR/$APP_NAME"

# Create App Folder Structure
mkdir -p "$APP_DIR/bin"
mkdir -p "$APP_DIR/assets"
mkdir -p "$APP_DIR/data"
mkdir -p "$APP_DIR/resources/profiles"

echo -e "${GREEN}📁 Assembling package structure...${NC}"

# 3. Copy Binaries
cp "$BIN_X86" "$APP_DIR/bin/crosssave-x86"
cp "$BIN_ARM" "$APP_DIR/bin/crosssave-arm"
chmod +x "$APP_DIR/bin/"*

# 3.1 Copy Shared Libraries (if available)
if [ -d "builds/linux/libs" ]; then
    echo -e "${GREEN}📚 Copying bundled libraries...${NC}"
    cp -r "builds/linux/libs" "$APP_DIR/"
fi

# 4. Copy Assets
# Icon
if [ -f "src-tauri/icons/128x128.png" ]; then
    cp "src-tauri/icons/128x128.png" "$APP_DIR/icon.png"
    cp "src-tauri/icons/128x128.png" "$APP_DIR/assets/icon.png"
else
    echo -e "${YELLOW}⚠️  Icon not found, using placeholder${NC}"
    touch "$APP_DIR/icon.png"
fi

# Resources (Profiles)
cp src-tauri/resources/profiles/*.json "$APP_DIR/resources/profiles/" 2>/dev/null || true

# 5. Create Launcher Script (mux_launch.sh INSIDE App Folder)
LAUNCHER="$APP_DIR/mux_launch.sh"
cat > "$LAUNCHER" << 'EOF'
#!/bin/sh
# HELP: CrossSave Cloud
# ICON: CrossSave Cloud
# GRID: CrossSave Cloud

. /opt/muos/script/var/func.sh

# Prevent multiple instances
if pgrep -f "crosssave-cloud" > /dev/null; then
    echo "App already running"
    exit 1
fi

echo "CrossSave Cloud" > /tmp/act_go

# Robustly get the current directory (where the script is)
DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="$DIR"
DATA_DIR="$APP_DIR/data"

# Ensure data directory exists
mkdir -p "$DATA_DIR"

cd "$APP_DIR" || exit

# Debug: Check libraries
echo "Checking libs in $APP_DIR/libs:" >> "$APP_DIR/log.txt"
ls -l "$APP_DIR/libs" >> "$APP_DIR/log.txt" 2>&1

# Detect Architecture
ARCH=$(uname -m)
if [ "$ARCH" = "aarch64" ]; then
    BINARY="./bin/crosssave-arm"
else
    BINARY="./bin/crosssave-x86"
fi

# Set Environment Variables for Tauri/Webkit
export HOME="$DATA_DIR"
export XDG_CONFIG_HOME="$DATA_DIR/config"
export XDG_DATA_HOME="$DATA_DIR/local/share"
export XDG_CACHE_HOME="$DATA_DIR/cache"
export LD_LIBRARY_PATH="$APP_DIR/libs:$LD_LIBRARY_PATH"

echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH" >> "$APP_DIR/log.txt"

# Stability Flags
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_FORCE_SANDBOX=0
export GDK_BACKEND=x11
export RUST_BACKTRACE=1

# Run Application
echo "Starting $BINARY..."
"$BINARY" > "$APP_DIR/log.txt" 2>&1

exit 0
EOF

chmod +x "$LAUNCHER"

# 6. Package
echo -e "${GREEN}📦 Packaging .muxapp...${NC}"
DATE=$(date +%Y%m%d)

# Get Version from tauri.conf.json
VERSION=$(grep -o '"version": "[^"]*"' src-tauri/tauri.conf.json | cut -d'"' -f4)
if [ -z "$VERSION" ]; then
    VERSION="0.0.1"
fi

# Use hyphens for filename, but keep space for internal folder
PACKAGE_FILENAME="CrossSave-Cloud-${DATE}-${VERSION}.muxapp"
FINAL_OUTPUT="$OUTPUT_DIR/$PACKAGE_FILENAME"

# Zip ONLY the App Folder (which contains mux_launch.sh)
cd "$WORK_DIR"
zip -q -r "$FINAL_OUTPUT" "$APP_NAME"

# 7. Cleanup
cd "$PROJECT_ROOT"
rm -rf "$WORK_DIR"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ MUOS Package Created!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "📍 File: ${YELLOW}builds/linux/MUOS/${PACKAGE_FILENAME}${NC}"
echo ""
echo -e "${BLUE}Checklist Verification:${NC}"
echo -e "  ✅ Structure: Single folder '${APP_NAME}' inside zip"
echo -e "  ✅ Launcher:  mux_launch.sh inside '${APP_NAME}'"
echo -e "  ✅ Headers:   #HELP, #ICON, #GRID included"
echo -e "  ✅ Install:   MUOS extracts to /MUOS/application/${APP_NAME}"
echo ""
