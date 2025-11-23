#!/bin/bash

# Build Docker image for Linux compilation
# Only need to run this once (or when you update dependencies)

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🐳 Building Docker Image for Linux Builds${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cd "$(dirname "$0")/.."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
  echo -e "${RED}❌ Error: Docker is not running${NC}"
  echo ""
  echo "Please start Docker Desktop and try again"
  exit 1
fi

echo -e "${YELLOW}This will take ~5 minutes (downloads Rust, GTK, etc.)${NC}"
echo ""

# Build image
docker build -f scripts/Dockerfile.linux -t crosssave-linux-builder .

echo ""
echo -e "${GREEN}✅ Docker image built successfully!${NC}"
echo ""
echo -e "${BLUE}Next step:${NC} Run ${GREEN}./scripts/build-linux-docker.sh${NC} to build your app"
echo ""
