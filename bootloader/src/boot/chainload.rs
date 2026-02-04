use crate::util::{Error, Result};
use uefi::boot;
use uefi::println;

/// Chainload image directly from memory buffer
///
/// This is simpler than writing to a file and loading from disk.
/// UEFI LoadImage supports loading directly from memory.
pub fn chainload_image(image_data: &[u8]) -> Result<()> {
    println!("Preparing to chainload image ({} bytes)...", image_data.len());

    // Load the image directly from memory buffer
    println!("  Loading image from memory...");
    let image_handle = unsafe {
        boot::load_image(
            boot::image_handle(),
            boot::LoadImageSource::FromBuffer {
                buffer: image_data,
                file_path: None,
            },
        )
        .map_err(|e| {
            println!("    Failed to load image: {:?}", e.status());
            Error::Uefi(e.status())
        })?
    };

    println!("  Image loaded successfully");
    println!();

    // Start the image (this should not return for Linux kernel)
    println!("===========================================");
    println!("Chainloading to boot image...");
    println!("===========================================");
    println!();

    unsafe {
        boot::start_image(image_handle).map_err(|e| {
            println!();
            println!("Failed to start image: {:?}", e.status());
            Error::Uefi(e.status())
        })?;
    }

    // If we get here, the image returned (shouldn't happen for Linux kernel)
    println!();
    println!("Warning: Image returned control to bootloader");
    Ok(())
}

