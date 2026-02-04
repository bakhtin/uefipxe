#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Creating ESP Image...${NC}"
echo "===================="

# Check if mtools is installed
if ! command -v mcopy &> /dev/null; then
    echo -e "${RED}Error: mtools not found${NC}"
    echo "Install with: sudo apt install mtools"
    exit 1
fi

# Check if mkfs.vfat is installed
if ! command -v mkfs.vfat &> /dev/null; then
    echo -e "${RED}Error: mkfs.vfat not found${NC}"
    echo "Install with: sudo apt install dosfstools"
    exit 1
fi

# Remove existing ESP image if it exists
if [ -f esp.img ]; then
    echo -e "${YELLOW}Removing existing ESP image...${NC}"
    rm esp.img
fi

# Create 256MB image
echo "Creating 256MB disk image..."
dd if=/dev/zero of=esp.img bs=1M count=256 status=progress

# Format as FAT32
echo "Formatting as FAT32..."
mkfs.vfat -F 32 esp.img

# Create directory structure
echo "Creating directory structure..."
mmd -i esp.img ::/EFI
mmd -i esp.img ::/EFI/BOOT
mmd -i esp.img ::/EFI/uefipxe
mmd -i esp.img ::/EFI/uefipxe/temp

# Create initial config file
echo "Creating initial config file..."
cat > /tmp/config.txt << 'EOF'
# UEFI PXE Bootloader Configuration
# Lines starting with # are comments

# Default image index (0-based)
default=0

# Image URLs (one per line)
# Example:
# url=https://boot.example.com/production.efi
# url=https://boot.example.com/staging.efi
EOF

# Copy config file to ESP
mcopy -i esp.img /tmp/config.txt ::/EFI/uefipxe/config.txt
rm /tmp/config.txt

# List contents
echo ""
echo -e "${GREEN}ESP image created successfully!${NC}"
echo "Contents:"
mdir -i esp.img -/ ::/

echo ""
echo "Image file: esp.img (256MB)"
echo -e "${GREEN}Done!${NC}"
