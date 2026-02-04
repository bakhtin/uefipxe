#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;
use uefi::{println, Status};

mod boot;
mod cli;
mod network;
mod storage;
mod util;

#[entry]
fn main() -> Status {
    // Initialize UEFI services (heap allocator, logger, panic handler)
    uefi::helpers::init().expect("Failed to initialize UEFI");

    // Initialize logger
    util::logger::init();

    // Print welcome message
    println!();
    println!("UEFI PXE Bootloader v{}", env!("CARGO_PKG_VERSION"));
    println!("=====================================");

    // Log startup
    util::logger::log_entry(log::Level::Info, "Bootloader started");

    // Load configuration
    let config = storage::load_config().unwrap_or_else(|e| {
        println!("Warning: Could not load config: {}", e);
        util::logger::log_entry(
            log::Level::Warn,
            &alloc::format!("Config load failed: {}, using empty config", e),
        );
        storage::Config::new()
    });
    storage::init_config(config);
    util::logger::log_entry(log::Level::Info, "Configuration loaded");

    // Run CLI REPL
    match cli::run() {
        Ok(_) => {
            println!("Exiting normally");
            util::logger::log_entry(log::Level::Info, "Bootloader exiting normally");
        }
        Err(e) => {
            println!("Error: {}", e);
            util::logger::log_entry(
                log::Level::Error,
                &alloc::format!("Bootloader error: {}", e),
            );
        }
    }

    Status::SUCCESS
}
