# UEFI PXE Bootloader

A UEFI bootloader written in Rust that downloads and chainloads Linux UKI (Unified Kernel Image) files over HTTPS.

## Features

- Interactive CLI for managing boot images
- Download UKI images over HTTPS with certificate validation
- Configuration stored on ESP (EFI System Partition)
- GCP metadata service integration
- Local QEMU testing support
- Google Cloud Platform deployment

## Requirements

### Build Requirements

- Rust nightly toolchain
- `rust-src` component
- `x86_64-unknown-uefi` target

### Testing Requirements

- QEMU (`qemu-system-x86_64`)
- OVMF UEFI firmware
- mtools (for FAT filesystem manipulation)
- dosfstools (for mkfs.vfat)

### Installation (Ubuntu/Debian)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install build dependencies
sudo apt install qemu-system-x86 ovmf mtools dosfstools

# Install Rust components (handled automatically by rust-toolchain.toml)
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
rustup target add x86_64-unknown-uefi --toolchain nightly
```

## Building

```bash
# Build the bootloader
./scripts/build.sh

# Output: target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi
```

## Testing Locally

```bash
# Run in QEMU (builds automatically if needed)
./scripts/qemu-test.sh

# Or run with a specific EFI file
./scripts/qemu-test.sh target/x86_64-unknown-uefi/release/uefipxe-bootloader.efi
```

## Project Structure

```
uefipxe/
├── Cargo.toml                    # Workspace root
├── .cargo/config.toml            # Build configuration
├── rust-toolchain.toml           # Rust toolchain specification
├── bootloader/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs               # Entry point
├── scripts/
│   ├── build.sh                  # Build script
│   ├── qemu-test.sh              # QEMU test runner
│   └── create-esp.sh             # Create ESP image
└── README.md
```

## CLI Commands (Planned)

| Command | Description |
|---------|-------------|
| `add <url>` | Add image URL to configuration |
| `remove <index>` | Remove image URL by index |
| `list` | Display all configured image URLs |
| `boot <index>` | Download and boot image |
| `default <index>` | Set default boot image |
| `save` | Write configuration to ESP |
| `test-network` | Test network connectivity |
| `logs` | Display buffered log messages |
| `exit` | Exit to firmware setup |

## Configuration

Configuration is stored in `\EFI\uefipxe\config.txt` on the ESP:

```
# UEFI PXE Bootloader Configuration
default=0
url=https://boot.example.com/production.efi
url=https://boot.example.com/staging.efi
```

## GCP Integration

On Google Cloud Platform, the bootloader can read configuration from instance metadata:

```bash
# Set metadata
gcloud compute instances add-metadata INSTANCE_NAME \
    --metadata=uefipxe-config='{
      "urls": [
        "https://boot.example.com/image1.efi",
        "https://boot.example.com/image2.efi"
      ],
      "default": 0
    }'
```

## Development Status

### Phase 1: Project Setup ✅ COMPLETE
- [x] Cargo workspace structure
- [x] Build configuration
- [x] Minimal UEFI application
- [x] QEMU testing scripts
- [x] ESP image creation

### Phase 2: CLI Infrastructure (In Progress)
- [ ] REPL loop
- [ ] Command parsing
- [ ] Basic commands (list, exit)
- [ ] Logging infrastructure

### Phase 3: Storage & Configuration (Planned)
- [ ] ESP filesystem access
- [ ] Config file parsing
- [ ] Configuration commands

### Phase 4: Network Stack (Planned)
- [ ] TCP/IP stack integration
- [ ] DHCP client
- [ ] DNS resolver

### Phase 5: HTTPS Support (Planned)
- [ ] TLS integration
- [ ] Certificate validation
- [ ] HTTP/HTTPS client

### Phase 6: GCP Metadata (Planned)
- [ ] Metadata service client
- [ ] Configuration merging

### Phase 7: Chainloading (Planned)
- [ ] Image download
- [ ] Memory management
- [ ] UKI chainloading

### Phase 8: Testing & Polish (Planned)
- [ ] Real hardware testing
- [ ] GCP testing
- [ ] Documentation

## License

MIT OR Apache-2.0

## References

- [uefi-rs Documentation](https://docs.rs/uefi/latest/uefi/)
- [UEFI Specification](https://uefi.org/specifications)
- [Linux EFI Stub](https://docs.kernel.org/admin-guide/efi-stub.html)
- [Unified Kernel Image](https://uapi-group.org/specifications/specs/unified_kernel_image/)
