use crate::util::{Error, Result};
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, SearchType};
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::{CStr16, Identify};
use heapless::Vec;

/// Path to the configuration file on the ESP
pub const CONFIG_PATH: &str = "\\EFI\\uefipxe\\config.txt";

/// Read a file from the ESP
pub fn read_file(path: &str) -> Result<Vec<u8, 8192>> {
    // Convert path to UCS-2
    let mut path_buf = [0u16; 256];
    let path_ucs2 = str_to_ucs2(path, &mut path_buf)?;

    // Locate the SimpleFileSystem protocol
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
        .map_err(|e| Error::Uefi(e.status()))?;

    // Try each handle until we find one that works
    for handle in &*handles {
        let result = try_read_from_handle(*handle, path_ucs2);
        if result.is_ok() {
            return result;
        }
    }

    Err(Error::NotFound)
}

/// Write a file to the ESP
pub fn write_file(path: &str, data: &[u8]) -> Result<()> {
    // Convert path to UCS-2
    let mut path_buf = [0u16; 256];
    let path_ucs2 = str_to_ucs2(path, &mut path_buf)?;

    // Locate the SimpleFileSystem protocol
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
        .map_err(|e| Error::Uefi(e.status()))?;

    // Try each handle until we find one that works
    for handle in &*handles {
        let result = try_write_to_handle(*handle, path_ucs2, data);
        if result.is_ok() {
            return result;
        }
    }

    Err(Error::NotFound)
}

/// Try to read a file from a specific filesystem handle
fn try_read_from_handle(handle: uefi::Handle, path: &CStr16) -> Result<Vec<u8, 8192>> {
    // Open the SimpleFileSystem protocol
    let mut fs = unsafe {
        boot::open_protocol::<SimpleFileSystem>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .map_err(|e| Error::Uefi(e.status()))?
    };

    // Open the root directory
    let mut root = fs.open_volume().map_err(|e| Error::Uefi(e.status()))?;

    // Open the file
    let file_handle = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .map_err(|e| Error::Uefi(e.status()))?;

    let mut file = match file_handle.into_type().map_err(|e| Error::Uefi(e.status()))? {
        uefi::proto::media::file::FileType::Regular(f) => f,
        uefi::proto::media::file::FileType::Dir(_) => return Err(Error::Io),
    };

    // Get file size
    let mut info_buf = [0u8; 256];
    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .map_err(|e| Error::Uefi(e.status()))?;

    let file_size = info.file_size() as usize;

    if file_size > 8192 {
        return Err(Error::BufferTooSmall);
    }

    // Read file contents
    let mut buffer = Vec::new();
    buffer.resize(file_size, 0).map_err(|_| Error::OutOfMemory)?;

    file.read(&mut buffer).map_err(|e| Error::Uefi(e.status()))?;

    Ok(buffer)
}

/// Try to write a file to a specific filesystem handle
fn try_write_to_handle(handle: uefi::Handle, path: &CStr16, data: &[u8]) -> Result<()> {
    // Open the SimpleFileSystem protocol
    let mut fs = unsafe {
        boot::open_protocol::<SimpleFileSystem>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .map_err(|e| Error::Uefi(e.status()))?
    };

    // Open the root directory
    let mut root = fs.open_volume().map_err(|e| Error::Uefi(e.status()))?;

    // Open/create the file
    let file_handle = root
        .open(
            path,
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .map_err(|e| Error::Uefi(e.status()))?;

    let mut file = match file_handle.into_type().map_err(|e| Error::Uefi(e.status()))? {
        uefi::proto::media::file::FileType::Regular(f) => f,
        uefi::proto::media::file::FileType::Dir(_) => return Err(Error::Io),
    };

    // Write data
    file.write(data).map_err(|e| Error::Uefi(e.status()))?;

    // Flush
    file.flush().map_err(|e| Error::Uefi(e.status()))?;

    Ok(())
}

/// Convert a Rust string to UCS-2 (UTF-16 without surrogates)
fn str_to_ucs2<'a>(s: &str, buf: &'a mut [u16]) -> Result<&'a CStr16> {
    if s.len() >= buf.len() {
        return Err(Error::BufferTooSmall);
    }

    let mut i = 0;
    for c in s.chars() {
        if i >= buf.len() - 1 {
            return Err(Error::BufferTooSmall);
        }
        buf[i] = c as u16;
        i += 1;
    }
    buf[i] = 0; // Null terminator

    // Safety: We just null-terminated the buffer
    unsafe { Ok(CStr16::from_u16_with_nul_unchecked(&buf[..=i])) }
}
