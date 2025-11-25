#!/bin/bash

# Build CrossSave Cloud for iOS (x86_64 simulator and aarch64 device)
# Must be run on macOS with Xcode installed

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if running on macOS
if [[ "$(uname -s)" != "Darwin" ]]; then
  echo -e "${RED}Error: This script must be run on macOS${NC}"
  echo ""
  echo "You are on: $(uname -s)"
  echo ""
  exit 1
fi

# Check for Xcode
if ! command -v xcodebuild &> /dev/null; then
  echo -e "${RED}Error: Xcode not found${NC}"
  echo "Install Xcode from the App Store"
  exit 1
fi

# Default values
ARCH="${ARCH:-all}"
DATE=$(date +%Y%m%d)

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --arch)
      ARCH="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --arch <x86_64|aarch64|all>   Target architecture (default: all)"
      echo "  --help                         Show this help message"
      echo ""
      echo "Architectures:"
      echo "  x86_64   - iOS Simulator (Intel Macs)"
      echo "  aarch64  - iOS Devices (iPhone/iPad)"
      echo "  all      - Build both"
      echo ""
      echo "Examples:"
      echo "  $0                    # Build both architectures"
      echo "  $0 --arch x86_64      # Build only simulator"
      echo "  $0 --arch aarch64     # Build only device"
      echo ""
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# If arch is "all", build both
if [[ "$ARCH" == "all" ]]; then
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}Building for all architectures${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  
  # Build x86_64 (simulator)
  "$0" --arch x86_64
  
  echo ""
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  
  # Build aarch64 (device)
  "$0" --arch aarch64
  
  echo ""
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}✅ All builds complete!${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  
  exit 0
fi

# Validate architecture
if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
  echo -e "${RED}Error: Invalid architecture '$ARCH'${NC}"
  echo "Supported: x86_64, aarch64, all"
  exit 1
fi

# Set target based on architecture
if [[ "$ARCH" == "x86_64" ]]; then
  TARGET="x86_64-apple-ios"
  BUILD_TYPE="simulator"
  TAURI_TARGET="x86_64-apple-ios"
else
  TARGET="aarch64-apple-ios"
  BUILD_TYPE="device"
  TAURI_TARGET="aarch64-apple-ios"
fi

# Output directory
OUTPUT_DIR="$(dirname "$0")/../builds/ios/$ARCH"
mkdir -p "$OUTPUT_DIR"

# Calculate version
EXISTING_BUILDS=$(ls "$OUTPUT_DIR"/crosssave-cloud-$DATE-* 2>/dev/null || true)
if [ -z "$EXISTING_BUILDS" ]; then
    VERSION="0.0.1"
else
    LAST_VERSION=$(echo "$EXISTING_BUILDS" | grep -o "0\.0\.[0-9]*" | grep -o "[0-9]*$" | sort -n | tail -1)
    NEXT_NUM=$((LAST_VERSION + 1))
    VERSION="0.0.$NEXT_NUM"
fi

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}CrossSave Cloud - iOS Build${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Architecture: ${YELLOW}$ARCH${NC}"
echo -e "Target:       ${YELLOW}$TARGET${NC}"
echo -e "Build Type:   ${YELLOW}$BUILD_TYPE${NC}"
echo -e "Version:      ${YELLOW}$VERSION${NC}"
echo -e "Date:         ${YELLOW}$DATE${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cd "$(dirname "$0")/.."

# Check dependencies
echo -e "${GREEN}📋 Checking dependencies...${NC}"

# Check for pnpm
if ! command -v pnpm &> /dev/null; then
  echo -e "${RED}Error: pnpm not found${NC}"
  echo "Install: npm install -g pnpm"
  exit 1
fi

# Check for Rust
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: Rust not found${NC}"
  echo "Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi

# Add target
if ! rustup target list --installed | grep -q "$TARGET"; then
  echo -e "${YELLOW}Installing Rust target: $TARGET${NC}"
  rustup target add "$TARGET"
else
  echo -e "${GREEN}✓ Rust target installed${NC}"
fi

# Build frontend
echo ""
echo -e "${GREEN}📦 Building frontend...${NC}"
pnpm install
pnpm build

# Initialize iOS project if needed
if [[ ! -d "src-tauri/gen/apple" ]]; then
  echo ""
  echo -e "${GREEN}🔧 Initializing iOS project...${NC}"
  cd src-tauri
  cargo tauri ios init
  cd ..
fi

# Build iOS app
echo ""
echo -e "${GREEN}🔨 Building iOS app...${NC}"

cd src-tauri

if [[ "$BUILD_TYPE" == "simulator" ]]; then
  # Build for simulator
  cargo tauri ios build --target "$TAURI_TARGET"
else
  # Build for device (requires code signing)
  echo -e "${YELLOW}Note: Device builds require code signing configuration${NC}"
  cargo tauri ios build --target "$TAURI_TARGET"
fi

cd ..

# Find the built app
if [[ "$BUILD_TYPE" == "simulator" ]]; then
  APP_PATH="src-tauri/gen/apple/build/x86_64-apple-ios-sim/debug/crosssave-cloud.app"
else
  APP_PATH="src-tauri/gen/apple/build/aarch64-apple-ios/debug/crosssave-cloud.app"
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo -e "${YELLOW}Warning: App bundle not found at expected location${NC}"
  echo "Searching for built app..."
  APP_PATH=$(find src-tauri/gen/apple/build -name "crosssave-cloud.app" -type d | head -1)
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo -e "${RED}Error: App bundle not found${NC}"
  exit 1
fi

echo ""
echo -e "${GREEN}📁 Packaging build...${NC}"

# Create IPA (for device builds)
if [[ "$BUILD_TYPE" == "device" ]]; then
  IPA_NAME="crosssave-cloud-$DATE-$VERSION-$ARCH.ipa"
  IPA_PATH="$OUTPUT_DIR/$IPA_NAME"
  
  # Create Payload directory
  PAYLOAD_DIR="$OUTPUT_DIR/Payload"
  mkdir -p "$PAYLOAD_DIR"
  
  # Copy app to Payload
  cp -r "$APP_PATH" "$PAYLOAD_DIR/"
  
  # Create IPA (zip file)
  (cd "$OUTPUT_DIR" && zip -r "$IPA_NAME" Payload)
  
  # Cleanup
  rm -rf "$PAYLOAD_DIR"
  
  echo -e "  ${GREEN}✓${NC} IPA: $IPA_PATH"
else
  # For simulator, just copy the app bundle
  APP_FINAL="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.app"
  cp -r "$APP_PATH" "$APP_FINAL"
  echo -e "  ${GREEN}✓${NC} App: $APP_FINAL"
fi

# Create tarball
TARBALL="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.tar.gz"
if [[ "$BUILD_TYPE" == "device" ]]; then
  tar -czf "$TARBALL" -C "$OUTPUT_DIR" "$IPA_NAME"
else
  tar -czf "$TARBALL" -C "$OUTPUT_DIR" "$(basename "$APP_FINAL")"
fi
echo -e "  ${GREEN}✓${NC} Tarball: $TARBALL"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Build complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "📦 Output: ${YELLOW}$OUTPUT_DIR${NC}"
echo -e "📅 Date: ${YELLOW}$DATE${NC}"
echo -e "🔢 Version: ${YELLOW}$VERSION${NC}"
echo ""
echo -e "${BLUE}Files:${NC}"
if [[ "$BUILD_TYPE" == "device" ]]; then
  echo -e "  IPA:     ${GREEN}$IPA_PATH${NC}"
else
  echo -e "  App:     ${GREEN}$APP_FINAL${NC}"
fi
echo -e "  Tarball: ${GREEN}$TARBALL${NC}"
echo ""

if [[ "$BUILD_TYPE" == "simulator" ]]; then
  echo -e "${BLUE}Testing on Simulator:${NC}"
  echo -e "  ${GREEN}xcrun simctl install booted \"$APP_FINAL\"${NC}"
  echo ""
elif [[ "$BUILD_TYPE" == "device" ]]; then
  echo -e "${BLUE}Installing on Device:${NC}"
  echo -e "  1. Use Xcode to install the IPA"
  echo -e "  2. Or use: ${GREEN}ios-deploy --bundle \"$IPA_PATH\"${NC}"
  echo ""
fi
