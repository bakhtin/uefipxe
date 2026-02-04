use crate::util::{Error, Result};
use uefi::boot::{self, SearchType};
use uefi::proto::network::snp::SimpleNetwork;
use uefi::{println, Guid, Handle, Identify};

/// DHCP4 Protocol GUID (from UEFI spec)
/// {8A219718-4EF5-4761-91C8-C0F04BDA9E56}
const DHCP4_PROTOCOL_GUID: Guid = Guid::from_bytes([
    0x18, 0x97, 0x21, 0x8a, 0xf5, 0x4e, 0x61, 0x47,
    0x91, 0xc8, 0xc0, 0xf0, 0x4b, 0xda, 0x9e, 0x56,
]);

/// IP4 Config2 Protocol GUID (from UEFI spec)
/// {5B446ED1-E30B-4FAA-871A-3654ECA36080}
const IP4_CONFIG2_PROTOCOL_GUID: Guid = Guid::from_bytes([
    0xd1, 0x6e, 0x44, 0x5b, 0x0b, 0xe3, 0xaa, 0x4f,
    0x87, 0x1a, 0x36, 0x54, 0xec, 0xa3, 0x60, 0x80,
]);

/// Initialize network interface with DHCP
pub fn initialize_network() -> Result<Handle> {
    println!("Initializing network...");

    // Find network interface handle
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleNetwork::GUID))
        .map_err(|e| Error::Uefi(e.status()))?;

    let nic_handle = handles
        .first()
        .copied()
        .ok_or(Error::NotFound)?;

    println!("  Found network interface");

    // Try to configure DHCP on this interface
    match crate::network::dhcp::configure_dhcp(nic_handle) {
        Ok(_) => {
            println!("  Network configured successfully via DHCP");
        }
        Err(e) => {
            println!("  DHCP configuration failed: {}", e);
            println!("  Continuing anyway - network might already be configured");
        }
    }

    println!("  Network initialization complete");

    Ok(nic_handle)
}

/// Simplified DHCP configuration attempt
/// This uses the DHCP4 Service Binding to create a child instance
fn configure_dhcp_simple(service_binding_handle: Handle) -> Result<()> {
    use uefi::boot::OpenProtocolAttributes;
    use uefi::boot::OpenProtocolParams;

    // Note: DHCP4 Service Binding protocol would need to be opened here
    // to create a child DHCP instance. This requires:
    // 1. Open DHCP4_SERVICE_BINDING_PROTOCOL
    // 2. Call CreateChild() to get a DHCP4 protocol instance
    // 3. Configure() the DHCP4 instance
    // 4. Start() DHCP to begin discovery
    //
    // This is complex and requires extensive unsafe code and uefi_raw protocol definitions.
    // For now, we document that DHCP is available but defer full implementation.

    println!("    (Full DHCP implementation requires unsafe protocol calls)");
    println!("    (This is a known limitation - use pre-configured network or UEFI shell)");

    // Return Ok to continue - the network might already be configured
    Ok(())
}

/// Try to get network status information
pub fn check_network_status() -> Result<()> {
    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&SimpleNetwork::GUID))
        .map_err(|e| Error::Uefi(e.status()))?;

    if handles.is_empty() {
        println!("No network interfaces found");
        return Err(Error::NotFound);
    }

    println!("Network Status:");
    println!("===============");
    println!("  Network interfaces: {}", handles.len());

    // Check if DHCP protocol is available
    match boot::locate_handle_buffer(SearchType::ByProtocol(&DHCP4_PROTOCOL_GUID)) {
        Ok(dhcp_handles) => {
            println!("  DHCP4 protocol: available ({} instances)", dhcp_handles.len());
        }
        Err(_) => {
            println!("  DHCP4 protocol: not available");
        }
    }

    // Check if IP4 Config2 protocol is available
    match boot::locate_handle_buffer(SearchType::ByProtocol(&IP4_CONFIG2_PROTOCOL_GUID)) {
        Ok(ip_handles) => {
            println!("  IP4Config2 protocol: available ({} instances)", ip_handles.len());
        }
        Err(_) => {
            println!("  IP4Config2 protocol: not available");
        }
    }

    Ok(())
}
