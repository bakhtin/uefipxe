use crate::util::{Error, Result};
use core::ptr;
use core::time::Duration;
use uefi::boot::{self, SearchType};
use uefi::{println, Guid, Handle, Status};
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::dhcp4::{
    Dhcp4ConfigData, Dhcp4ModeData, Dhcp4Protocol, Dhcp4State,
};

/// DHCP4 Service Binding Protocol GUID
const DHCP4_SERVICE_BINDING_GUID: Guid = Guid::from_bytes([
    0xd8, 0x39, 0x9a, 0x9d, 0x42, 0xbd, 0x73, 0x4a,
    0xa4, 0xd5, 0x8e, 0xe9, 0x4b, 0xe1, 0x13, 0x80,
]);

/// DHCP4 Protocol GUID
const DHCP4_PROTOCOL_GUID: Guid = Guid::from_bytes([
    0x18, 0x97, 0x21, 0x8a, 0xf5, 0x4e, 0x61, 0x47,
    0x91, 0xc8, 0xc0, 0xf0, 0x4b, 0xda, 0x9e, 0x56,
]);

/// Configure DHCP on a network interface
pub fn configure_dhcp(nic_handle: Handle) -> Result<()> {
    println!("  Configuring DHCP...");

    // Step 1: Locate DHCP4 Service Binding Protocol handles
    let service_handles = boot::locate_handle_buffer(SearchType::ByProtocol(&DHCP4_SERVICE_BINDING_GUID))
        .map_err(|e| {
            println!("    Failed to locate DHCP4 Service Binding: {:?}", e.status());
            Error::Uefi(e.status())
        })?;

    if service_handles.is_empty() {
        println!("    No DHCP4 Service Binding found");
        return Err(Error::NotFound);
    }

    println!("    Found {} DHCP4 Service Binding instance(s)", service_handles.len());

    // Use the first service binding handle
    let service_handle = service_handles[0];

    // Step 2: Get Service Binding Protocol interface
    let mut service_binding_ptr: *mut ServiceBindingProtocol = ptr::null_mut();

    let status = unsafe {
        let system_table = uefi::table::system_table_raw().unwrap();
        let boot_services = (*system_table.as_ptr()).boot_services;
        ((*boot_services).open_protocol)(
            service_handle.as_ptr(),
            &DHCP4_SERVICE_BINDING_GUID as *const Guid as *const uefi_raw::Guid,
            &mut service_binding_ptr as *mut *mut ServiceBindingProtocol as *mut *mut core::ffi::c_void,
            boot::image_handle().as_ptr(),
            ptr::null_mut(),
            0x02, // GET_PROTOCOL
        )
    };

    if status.is_error() {
        println!("    Failed to open Service Binding Protocol: {:?}", status);
        return Err(Error::Uefi(status));
    }

    println!("    Opened Service Binding Protocol");

    // Step 3: Create DHCP4 child instance
    let mut child_handle_raw: uefi_raw::Handle = ptr::null_mut();

    let status = unsafe {
        ((*service_binding_ptr).create_child)(
            service_binding_ptr,
            &mut child_handle_raw as *mut uefi_raw::Handle as *mut *mut core::ffi::c_void
        )
    };

    if status.is_error() {
        println!("    Failed to create DHCP4 child: {:?}", status);
        return Err(Error::Uefi(status));
    }

    let child_handle = unsafe { Handle::from_ptr(child_handle_raw) }.ok_or(Error::Unknown)?;
    println!("    Created DHCP4 child instance");

    // Step 4: Open DHCP4 Protocol on child handle
    let mut dhcp4_ptr: *mut Dhcp4Protocol = ptr::null_mut();

    let status = unsafe {
        let system_table = uefi::table::system_table_raw().unwrap();
        let boot_services = (*system_table.as_ptr()).boot_services;
        ((*boot_services).open_protocol)(
            child_handle.as_ptr(),
            &DHCP4_PROTOCOL_GUID as *const Guid as *const uefi_raw::Guid,
            &mut dhcp4_ptr as *mut *mut Dhcp4Protocol as *mut *mut core::ffi::c_void,
            boot::image_handle().as_ptr(),
            ptr::null_mut(),
            0x02, // GET_PROTOCOL
        )
    };

    if status.is_error() {
        println!("    Failed to open DHCP4 Protocol: {:?}", status);
        return Err(Error::Uefi(status));
    }

    println!("    Opened DHCP4 Protocol");

    // Step 5: Configure DHCP4
    let config = create_default_dhcp_config();

    let status = unsafe {
        ((*dhcp4_ptr).configure)(dhcp4_ptr, &config)
    };

    if status.is_error() {
        println!("    Failed to configure DHCP4: {:?}", status);
        return Err(Error::Uefi(status));
    }

    println!("    DHCP4 configured");

    // Step 6: Start DHCP discovery (synchronous, no event)
    let status = unsafe {
        ((*dhcp4_ptr).start)(dhcp4_ptr, ptr::null_mut())
    };

    if status.is_error() {
        println!("    Failed to start DHCP4: {:?}", status);
        return Err(Error::Uefi(status));
    }

    println!("    DHCP4 discovery started");

    // Step 7: Poll for DHCP completion
    let result = poll_dhcp_completion(dhcp4_ptr, Duration::from_secs(30));

    match result {
        Ok(ip_addr) => {
            println!("    DHCP completed successfully");
            println!("    Assigned IP: {}.{}.{}.{}",
                ip_addr[0], ip_addr[1], ip_addr[2], ip_addr[3]);
            Ok(())
        }
        Err(e) => {
            println!("    DHCP failed: {}", e);
            Err(e)
        }
    }
}

/// Create default DHCP configuration
fn create_default_dhcp_config() -> Dhcp4ConfigData {
    Dhcp4ConfigData {
        discover_try_count: 4,
        discover_timeout: ptr::null_mut(),
        request_try_count: 4,
        request_timeout: ptr::null_mut(),
        client_address: uefi_raw::Ipv4Address([0, 0, 0, 0]),
        callback: None,
        callback_context: ptr::null_mut(),
        option_count: 0,
        option_list: ptr::null_mut(),
    }
}

/// Poll DHCP state until BOUND or timeout
fn poll_dhcp_completion(
    dhcp4_ptr: *mut Dhcp4Protocol,
    timeout: Duration,
) -> Result<[u8; 4]> {
    let timeout_ms = timeout.as_millis() as u64;
    let poll_interval_ms = 100;
    let max_polls = timeout_ms / poll_interval_ms;

    for _poll_count in 0..max_polls {
        // Get current DHCP state
        let mut mode_data: Dhcp4ModeData = unsafe { core::mem::zeroed() };

        let status = unsafe {
            ((*dhcp4_ptr).get_mode_data)(dhcp4_ptr, &mut mode_data)
        };

        if status.is_error() {
            println!("    Failed to get DHCP mode data: {:?}", status);
            return Err(Error::Uefi(status));
        }

        // Check state
        match mode_data.state {
            Dhcp4State::BOUND => {
                // Success!
                return Ok(mode_data.client_address.0);
            }
            Dhcp4State::INIT | Dhcp4State::SELECTING | Dhcp4State::REQUESTING => {
                // Still in progress
                boot::stall(Duration::from_millis(poll_interval_ms));
            }
            _ => {
                println!("    DHCP entered unexpected state: {:?}", mode_data.state);
                return Err(Error::Unknown);
            }
        }
    }

    println!("    DHCP timeout after {} seconds", timeout.as_secs());
    Err(Error::Unknown)
}
