#!/bin/bash

# Build CrossSave Cloud for Windows (x86_64 and aarch64)
# Can be run on Windows (WSL/MSYS2), Linux, or macOS with cross-compilation

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
      echo "  $0 --arch aarch64     # Build only ARM64"
      echo ""
      echo "Note: Cross-compilation requires mingw-w64 toolchain"
      echo "  Linux: sudo apt-get install mingw-w64"
      echo "  macOS: brew install mingw-w64"
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
  if rustup target list --installed | grep -q "aarch64-pc-windows-msvc"; then
    "$0" --arch aarch64
  else
    echo -e "${YELLOW}⚠ Skipping aarch64 build (target not installed)${NC}"
    echo "Install: rustup target add aarch64-pc-windows-msvc"
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

# Set target based on host OS and target architecture
HOST_OS=$(uname -s)
if [[ "$HOST_OS" == "MINGW"* ]] || [[ "$HOST_OS" == "MSYS"* ]] || [[ "$HOST_OS" == "CYGWIN"* ]]; then
  # Running on Windows
  TARGET="$ARCH-pc-windows-msvc"
else
  # Cross-compiling from Linux/macOS
  TARGET="$ARCH-pc-windows-gnu"
fi

# Output directory
OUTPUT_DIR="$(dirname "$0")/../builds/windows/$ARCH"
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
echo -e "${GREEN}CrossSave Cloud - Windows Build${NC}"
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

# Check for cross-compiler if needed (for GNU target)
if [[ "$TARGET" == *"-gnu" ]]; then
  if [[ "$ARCH" == "x86_64" ]] && ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${YELLOW}Warning: mingw-w64 not found, build may fail${NC}"
    echo "Install: sudo apt-get install mingw-w64 (Linux) or brew install mingw-w64 (macOS)"
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
if [[ "$TARGET" == "x86_64-pc-windows-gnu" ]]; then
  export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
  export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
  export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
  export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
fi

cd src-tauri
cargo build --release --target "$TARGET"
cd ..

# Find binary
BINARY="src-tauri/target/$TARGET/release/crosssave-cloud.exe"

if [[ ! -f "$BINARY" ]]; then
  echo -e "${RED}Error: Binary not found${NC}"
  exit 1
fi

echo ""
echo -e "${GREEN}📁 Packaging build...${NC}"

# Copy binary
FINAL_BINARY="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.exe"
cp "$BINARY" "$FINAL_BINARY"
echo -e "  ${GREEN}✓${NC} Binary: $FINAL_BINARY"

# Strip to reduce size (if available)
if command -v strip &> /dev/null; then
  strip "$FINAL_BINARY" 2>/dev/null || true
  echo -e "  ${GREEN}✓${NC} Stripped binary"
fi

# Create zip archive
ZIP_FILE="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.zip"
(cd "$OUTPUT_DIR" && zip -q "$(basename "$ZIP_FILE")" "$(basename "$FINAL_BINARY")")
echo -e "  ${GREEN}✓${NC} Zip: $ZIP_FILE"

# Create MSI installer if on Windows with WiX
if [[ "$HOST_OS" == "MINGW"* ]] || [[ "$HOST_OS" == "MSYS"* ]]; then
  if command -v candle &> /dev/null; then
    echo ""
    echo -e "${GREEN}📦 Creating MSI installer...${NC}"
    
    cd src-tauri
    cargo tauri build --target "$TARGET" --bundles msi
    cd ..
    
    # Find and copy MSI
    MSI_SOURCE="src-tauri/target/$TARGET/release/bundle/msi/crosssave-cloud_0.0.1_$ARCH.msi"
    if [[ -f "$MSI_SOURCE" ]]; then
      MSI_FINAL="$OUTPUT_DIR/crosssave-cloud-$DATE-$VERSION-$ARCH.msi"
      cp "$MSI_SOURCE" "$MSI_FINAL"
      echo -e "  ${GREEN}✓${NC} MSI: $MSI_FINAL"
    fi
  fi
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Build complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "📦 Output: ${YELLOW}$OUTPUT_DIR${NC}"
echo -e "📅 Date: ${YELLOW}$DATE${NC}"
echo -e "🔢 Version: ${YELLOW}$VERSION${NC}"
echo ""
echo -e "${BLUE}Files:${NC}"
echo -e "  Binary: ${GREEN}$FINAL_BINARY${NC}"
echo -e "  Zip:    ${GREEN}$ZIP_FILE${NC}"
if [[ -f "$MSI_FINAL" ]]; then
  echo -e "  MSI:    ${GREEN}$MSI_FINAL${NC}"
fi
echo ""
echo -e "${BLUE}Usage:${NC}"
echo -e "  ${GREEN}crosssave-cloud-$DATE-$VERSION-$ARCH.exe${NC}"
echo ""
