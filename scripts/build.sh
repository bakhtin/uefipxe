#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building UEFI PXE Bootloader...${NC}"
echo "================================"

# Check if rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    echo "Install Rust from https://rustup.rs/"
    exit 1
fi

# Check if nightly toolchain is installed
if ! rustup toolchain list | grep -q "nightly"; then
    echo -e "${YELLOW}Installing nightly toolchain...${NC}"
    rustup toolchain install nightly
fi

# Check if rust-src component is installed (required for build-std)
if ! rustup component list --installed --toolchain nightly | grep -q "rust-src"; then
    echo -e "${YELLOW}Installing rust-src component...${NC}"
    rustup component add rust-src --toolchain nightly
fi

# NOTE: We use build-std, so we do NOT install the x86_64-unknown-uefi target
# Installing precompiled libraries would conflict with build-std

# Build the bootloader
echo -e "${YELLOW}Building bootloader...${NC}"
cargo +nightly build \
    --target x86_64-unknown-uefi \
    --release \
    --quiet

# Check if build succeeded
if [ -f target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi ]; then
    SIZE=$(du -h target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi | cut -f1)
    echo -e "${GREEN}Build successful!${NC}"
    echo "Output: target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi"
    echo "Size: $SIZE"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi
