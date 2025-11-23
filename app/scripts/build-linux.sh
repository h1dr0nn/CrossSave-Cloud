#!/bin/bash

# Build CrossSave Cloud for Linux (x86_64 and aarch64)
# Must be run on a Linux machine

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if running on Linux
if [[ "$(uname -s)" != "Linux" ]]; then
  echo -e "${RED}Error: This script must be run on Linux${NC}"
  echo ""
  echo "You are on: $(uname -s)"
  echo ""
  echo "Options:"
  echo "  1. Run this script on a Linux machine"
  echo "  2. Use WSL2 on Windows"
  echo "  3. Use a Linux VM"
  echo ""
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
      echo "Examples:"
      echo "  $0                    # Build both architectures"
      echo "  $0 --arch x86_64      # Build only x86_64"
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
  
  # Build x86_64
  "$0" --arch x86_64
  
  echo ""
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  
  # Build aarch64 if cross-compile available
  if command -v aarch64-linux-gnu-gcc &> /dev/null || [[ "$(uname -m)" == "aarch64" ]]; then
    "$0" --arch aarch64
  else
    echo -e "${YELLOW}⚠ Skipping aarch64 build (cross-compiler not found)${NC}"
    echo "Install: sudo apt-get install gcc-aarch64-linux-gnu"
  fi
  
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

# Set target
TARGET="$ARCH-unknown-linux-gnu"

# Output directory
OUTPUT_DIR="$(dirname "$0")/../builds/linux/$ARCH"
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
echo -e "${GREEN}CrossSave Cloud - Linux Build${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Architecture: ${YELLOW}$ARCH${NC}"
echo -e "Target:       ${YELLOW}$TARGET${NC}"
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

# Check for cross-compiler if needed
HOST_ARCH=$(uname -m)
if [[ "$HOST_ARCH" != "$ARCH" ]]; then
  if [[ "$ARCH" == "aarch64" ]] && ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo -e "${RED}Error: Cross-compiler not found${NC}"
    echo "Install: sudo apt-get install gcc-aarch64-linux-gnu"
    exit 1
  fi
fi

# Build frontend
echo ""
echo -e "${GREEN}📦 Building frontend...${NC}"
pnpm install
pnpm build

# Build backend
echo ""
echo -e "${GREEN}🔨 Building Rust backend...${NC}"

# Set cross-compile env if needed
if [[ "$ARCH" == "aarch64" && "$HOST_ARCH" != "aarch64" ]]; then
  export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
  export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
  export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
fi

cd src-tauri
cargo build --release --target "$TARGET"
cd ..

# Find binary
BINARY="src-tauri/target/$TARGET/release/crosssave-cloud"

if [[ ! -f "$BINARY" ]]; then
  echo -e "${RED}Error: Binary not found${NC}"
  exit 1
fi

echo ""
echo -e "${GREEN}📁 Packaging build...${NC}"

# Copy binary
FINAL_BINARY="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH"
cp "$BINARY" "$FINAL_BINARY"
chmod +x "$FINAL_BINARY"
echo -e "  ${GREEN}✓${NC} Binary: $FINAL_BINARY"

# Strip to reduce size
if command -v strip &> /dev/null; then
  strip "$FINAL_BINARY"
  echo -e "  ${GREEN}✓${NC} Stripped binary"
fi

# Create tarball
TARBALL="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.tar.gz"
tar -czf "$TARBALL" -C "$OUTPUT_DIR" "$(basename "$FINAL_BINARY")"
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
echo -e "  Binary:  ${GREEN}$FINAL_BINARY${NC}"
echo -e "  Tarball: ${GREEN}$TARBALL${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo -e "  ${GREEN}./crosssave-cloud-$DATE-$VERSION-$ARCH${NC}"
echo ""
