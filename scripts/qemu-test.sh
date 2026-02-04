#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}UEFI PXE Bootloader - QEMU Test${NC}"
echo "=================================="

# Check if EFI file is provided as argument, otherwise build
if [ -z "$1" ]; then
    echo -e "${YELLOW}No EFI file specified, building...${NC}"
    ./scripts/build.sh
    EFI_FILE="target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi"
else
    EFI_FILE="$1"
fi

# Check if EFI file exists
if [ ! -f "$EFI_FILE" ]; then
    echo -e "${RED}Error: EFI file not found: $EFI_FILE${NC}"
    exit 1
fi

echo "Using EFI file: $EFI_FILE"

# Create ESP image if needed
if [ ! -f esp.img ]; then
    echo -e "${YELLOW}ESP image not found, creating...${NC}"
    ./scripts/create-esp.sh
fi

# Copy bootloader to ESP
echo "Copying bootloader to ESP..."
mcopy -i esp.img -o "$EFI_FILE" ::/EFI/BOOT/BOOTX64.EFI

# Check for OVMF firmware (try different locations)
OVMF_CODE=""
OVMF_VARS_SRC=""

if [ -f "/usr/share/OVMF/OVMF_CODE.fd" ]; then
    OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
    OVMF_VARS_SRC="/usr/share/OVMF/OVMF_VARS.fd"
elif [ -f "/usr/share/OVMF/OVMF_CODE_4M.fd" ]; then
    OVMF_CODE="/usr/share/OVMF/OVMF_CODE_4M.fd"
    OVMF_VARS_SRC="/usr/share/OVMF/OVMF_VARS_4M.fd"
elif [ -f "/usr/share/ovmf/OVMF.fd" ]; then
    OVMF_CODE="/usr/share/ovmf/OVMF.fd"
    OVMF_VARS_SRC="/usr/share/ovmf/OVMF.fd"
else
    echo -e "${RED}Error: OVMF firmware not found${NC}"
    echo "Install with: sudo apt install ovmf"
    exit 1
fi

echo "Using OVMF firmware: $OVMF_CODE"

# Create OVMF vars if needed
if [ ! -f OVMF_VARS.fd ]; then
    echo "Creating OVMF variables file..."
    cp "$OVMF_VARS_SRC" OVMF_VARS.fd
fi

# Check if QEMU is installed
if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo -e "${RED}Error: qemu-system-x86_64 not found${NC}"
    echo "Install with: sudo apt install qemu-system-x86"
    exit 1
fi

echo -e "${GREEN}Starting QEMU...${NC}"
echo "Press Ctrl+C to exit"
echo ""

# Run QEMU (use -nographic for headless, or -display gtk for GUI)
qemu-system-x86_64 \
    -enable-kvm \
    -m 4096M \
    -drive if=pflash,format=raw,readonly=on,file="${OVMF_CODE}" \
    -drive if=pflash,format=raw,file=OVMF_VARS.fd \
    -drive format=raw,file=esp.img \
    -netdev user,id=net0 \
    -device virtio-net-pci,netdev=net0 \
    -nographic
