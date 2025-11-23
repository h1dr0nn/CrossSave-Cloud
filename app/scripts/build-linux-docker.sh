#!/bin/bash

# Build CrossSave Cloud for Linux using Docker
# Run on macOS to build Linux binaries

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🐳 CrossSave Cloud - Docker Linux Build${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cd "$(dirname "$0")/.."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
  echo -e "${RED}❌ Error: Docker is not running${NC}"
  echo ""
  echo "Please install and start Docker Desktop:"
  echo "  1. Download from: https://www.docker.com/products/docker-desktop"
  echo "  2. Install Docker Desktop"
  echo "  3. Open Docker Desktop app"
  echo "  4. Wait for it to start (whale icon in menu bar)"
  echo "  5. Run this script again"
  echo ""
  exit 1
fi

echo -e "${GREEN}✓ Docker is running${NC}"
echo ""

# Check if image exists
if ! docker images | grep -q "crosssave-linux-builder"; then
  echo -e "${RED}❌ Error: Docker image not found${NC}"
  echo ""
  echo "Please build the Docker image first:"
  echo -e "  ${GREEN}./scripts/build-docker-image.sh${NC}"
  echo ""
  exit 1
fi

echo -e "${GREEN}✓ Docker image found${NC}"
echo ""

# Run build in container
echo -e "${GREEN}🔨 Building in Docker container...${NC}"
echo ""

# Build x86_64
echo -e "${BLUE}Building x86_64 (native)...${NC}"
docker run --rm \
  -v "$(pwd)":/app \
  -v /app/node_modules \
  -v /app/.pnpm-store \
  -w /app \
  -e CI=true \
  crosssave-linux-builder \
  bash -c "
    set -e
    
    echo '📦 Installing dependencies...'
    pnpm install
    
    echo ''
    echo '🏗️  Building frontend...'
    pnpm build
    
    echo ''
    echo '🔨 Building Rust backend (x86_64 native)...'
    cd src-tauri
    cargo build --release
    cd ..
    
    echo ''
    echo '✅ x86_64 build complete!'
  "

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Build aarch64
echo -e "${BLUE}Building aarch64...${NC}"
docker run --rm \
  -v "$(pwd)":/app \
  -w /app \
  -e CI=true \
  crosssave-linux-builder \
  bash -c "
    set -e
    
    echo '📦 Installing ARM64 cross-compiler...'
    apt-get update -qq
    apt-get install -y -qq gcc-aarch64-linux-gnu
    
    echo ''
    echo '📦 Installing Rust target...'
    rustup target add aarch64-unknown-linux-gnu
    
    echo ''
    echo '🔨 Building aarch64...'
    cd src-tauri
    
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
    export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    
    rustup target add aarch64-unknown-linux-gnu
    cargo build --release --target aarch64-unknown-linux-gnu
    cd ..
    
    echo ''
    echo '✅ aarch64 build complete!'
  "

# Process both builds
DATE=$(date +%Y%m%d)

# Calculate version
OUTPUT_DIR_X86="builds/linux/x86_64"
mkdir -p "$OUTPUT_DIR_X86"

EXISTING_BUILDS=$(ls "$OUTPUT_DIR_X86"/crosssave-cloud-$DATE-* 2>/dev/null || true)
if [ -z "$EXISTING_BUILDS" ]; then
    VERSION="0.0.1"
else
    LAST_VERSION=$(echo "$EXISTING_BUILDS" | grep -o "0\.0\.[0-9]*" | grep -o "[0-9]*$" | sort -n | tail -1)
    NEXT_NUM=$((LAST_VERSION + 1))
    VERSION="0.0.$NEXT_NUM"
fi

echo ""
echo -e "${GREEN}📁 Packaging builds...${NC}"

# Package x86_64
BINARY_X86="src-tauri/target/release/crosssave-cloud"
if [[ -f "$BINARY_X86" ]]; then
  FINAL_X86="$OUTPUT_DIR_X86/crosssave-cloud-$DATE-$VERSION-x86_64"
  cp "$BINARY_X86" "$FINAL_X86"
  chmod +x "$FINAL_X86"
  
  TARBALL_X86="$OUTPUT_DIR_X86/crosssave-cloud-$DATE-$VERSION-x86_64.tar.gz"
  tar -czf "$TARBALL_X86" -C "$OUTPUT_DIR_X86" "$(basename "$FINAL_X86")"
  
  echo -e "  ${GREEN}✓${NC} x86_64:  $FINAL_X86"
fi

# Package aarch64
OUTPUT_DIR_ARM="builds/linux/aarch64"
mkdir -p "$OUTPUT_DIR_ARM"

BINARY_ARM="src-tauri/target/aarch64-unknown-linux-gnu/release/crosssave-cloud"
if [[ -f "$BINARY_ARM" ]]; then
  FINAL_ARM="$OUTPUT_DIR_ARM/crosssave-cloud-$DATE-$VERSION-aarch64"
  cp "$BINARY_ARM" "$FINAL_ARM"
  chmod +x "$FINAL_ARM"
  
  TARBALL_ARM="$OUTPUT_DIR_ARM/crosssave-cloud-$DATE-$VERSION-aarch64.tar.gz"
  tar -czf "$TARBALL_ARM" -C "$OUTPUT_DIR_ARM" "$(basename "$FINAL_ARM")"
  
  echo -e "  ${GREEN}✓${NC} aarch64: $FINAL_ARM"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All builds complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "📦 Output:"
echo -e "   x86_64:  ${GREEN}builds/linux/x86_64/${NC}"
echo -e "   aarch64: ${GREEN}builds/linux/aarch64/${NC}"
echo -e "📅 Date:    ${YELLOW}$DATE${NC}"
echo -e "🔢 Version: ${YELLOW}$VERSION${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo -e "  x86_64 (PC/Batocera):     ${GREEN}./crosssave-cloud-$DATE-$VERSION-x86_64${NC}"
echo -e "  aarch64 (Pi/Handhelds):   ${GREEN}./crosssave-cloud-$DATE-$VERSION-aarch64${NC}"
echo ""
