use crate::util::{Error, Result};
use alloc::vec::Vec;
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, SearchType};
use uefi::proto::network::http::HttpHelper;
use uefi::proto::network::snp::SimpleNetwork;
use uefi::{println, Identify};
use uefi_raw::protocol::network::http::HttpStatusCode;

/// Download a file over HTTP
pub fn download(url: &str) -> Result<Vec<u8>> {
    println!("Downloading: {}", url);

    // Initialize network (attempts DHCP configuration if available)
    let nic_handle = crate::network::init::initialize_network()?;

    // Create HTTP helper
    println!("  Initializing HTTP...");
    let mut http_helper = HttpHelper::new(nic_handle).map_err(|e| Error::Uefi(e.status()))?;

    // Configure HTTP protocol with defaults (IPv4, HTTP/1.0, 10s timeout)
    println!("  Configuring HTTP...");
    http_helper
        .configure()
        .map_err(|e| Error::Uefi(e.status()))?;

    // Send GET request
    println!("  Sending request...");
    http_helper
        .request_get(url)
        .map_err(|e| Error::Uefi(e.status()))?;

    // Receive response (expect body data)
    println!("  Receiving response...");
    let response = http_helper
        .response_first(true)
        .map_err(|e| Error::Uefi(e.status()))?;

    // Check HTTP status code
    if response.status != HttpStatusCode::STATUS_200_OK {
        println!("  HTTP error: status code {:?}", response.status);
        return Err(Error::Io);
    }

    // Start with initial body chunk
    let mut data = response.body;
    println!("  Downloaded {} bytes (initial chunk)", data.len());

    // Get remaining chunks for larger files
    // Only print progress every 10 chunks (~14KB) to reduce output
    let mut chunk_count = 0;
    const PROGRESS_INTERVAL: usize = 10;

    loop {
        match http_helper.response_more() {
            Ok(chunk) => {
                if chunk.is_empty() {
                    break; // No more data
                }
                data.extend_from_slice(&chunk);
                chunk_count += 1;

                // Print progress every N chunks
                if chunk_count % PROGRESS_INTERVAL == 0 {
                    println!("  Progress: {} bytes", data.len());
                }
            }
            Err(_) => break, // No more data or error
        }
    }

    println!("  Download complete: {} bytes total", data.len());
    Ok(data)
}

/// Test if network is available
pub fn test_network() -> Result<()> {
    // Check if we have a network interface
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleNetwork::GUID))
        .map_err(|e| Error::Uefi(e.status()))?;

    if handles.is_empty() {
        println!("No network interfaces found");
        return Err(Error::NotFound);
    }

    println!("Found {} network interface(s)", handles.len());

    // Get MAC address of first interface
    if let Some(&handle) = handles.first() {
        let snp = unsafe {
            boot::open_protocol::<SimpleNetwork>(
                OpenProtocolParams {
                    handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .map_err(|e| Error::Uefi(e.status()))?
        };

        let mode = snp.mode();
        let mac = mode.current_address.0;
        println!("MAC address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    }

    Ok(())
}
