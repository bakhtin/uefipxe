use crate::storage;
use crate::util::{Error, Result};
use heapless::String;

const MAX_URL_LEN: usize = 256;

/// Available CLI commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Display help
    Help,
    /// List all configured image URLs
    List,
    /// Add a new image URL
    Add(String<MAX_URL_LEN>),
    /// Remove an image URL by index
    Remove(usize),
    /// Boot an image by index
    Boot(usize),
    /// Set default boot image
    Default(usize),
    /// Save configuration to ESP
    Save,
    /// Test network connectivity
    TestNetwork,
    /// Display log messages
    Logs,
    /// Exit to firmware
    Exit,
}

impl Command {
    /// Execute the command
    pub fn execute(&self) -> Result<()> {
        match self {
            Command::Help => {
                Self::print_help();
                Ok(())
            }
            Command::List => Self::exec_list(),
            Command::Add(url) => Self::exec_add(url),
            Command::Remove(index) => Self::exec_remove(*index),
            Command::Boot(index) => Self::exec_boot(*index),
            Command::Default(index) => Self::exec_default(*index),
            Command::Save => Self::exec_save(),
            Command::TestNetwork => Self::exec_test_network(),
            Command::Logs => Self::exec_logs(),
            Command::Exit => Self::exec_exit(),
        }
    }

    fn print_help() {
        uefi::println!();
        uefi::println!("Available Commands:");
        uefi::println!("==================");
        uefi::println!("  help                 - Display this help message");
        uefi::println!("  list                 - List all configured image URLs");
        uefi::println!("  add <url>            - Add a new image URL");
        uefi::println!("  remove <index>       - Remove image URL by index");
        uefi::println!("  boot <index>         - Download and boot image");
        uefi::println!("  default <index>      - Set default boot image");
        uefi::println!("  save                 - Save configuration to ESP");
        uefi::println!("  test-network         - Test network connectivity");
        uefi::println!("  logs                 - Display buffered log messages");
        uefi::println!("  exit                 - Exit to firmware setup");
        uefi::println!();
    }

    fn exec_list() -> Result<()> {
        let config = storage::get_config().ok_or(Error::Unknown)?;

        uefi::println!();
        uefi::println!("Configured Images:");
        uefi::println!("==================");

        if config.urls.is_empty() {
            uefi::println!("  (no images configured)");
        } else {
            for (i, url) in config.urls.iter().enumerate() {
                let default_marker = if config.default_index == Some(i) {
                    " [DEFAULT]"
                } else {
                    ""
                };
                uefi::println!("  [{}] {}{}", i, url, default_marker);
            }
        }

        uefi::println!();
        Ok(())
    }

    fn exec_add(url: &str) -> Result<()> {
        let config = storage::get_config_mut().ok_or(Error::Unknown)?;

        config.add_url(url)?;

        uefi::println!("Added: {}", url);
        uefi::println!("Total images: {}", config.urls.len());
        uefi::println!("Remember to run 'save' to persist changes to ESP");

        Ok(())
    }

    fn exec_remove(index: usize) -> Result<()> {
        let config = storage::get_config_mut().ok_or(Error::Unknown)?;

        if index >= config.urls.len() {
            uefi::println!("Error: Index {} out of range (max: {})", index, config.urls.len() - 1);
            return Err(Error::NotFound);
        }

        let url = config.urls[index].clone();
        config.remove_url(index)?;

        uefi::println!("Removed: {}", url);
        uefi::println!("Total images: {}", config.urls.len());
        uefi::println!("Remember to run 'save' to persist changes to ESP");

        Ok(())
    }

    fn exec_boot(index: usize) -> Result<()> {
        let config = storage::get_config().ok_or(Error::Unknown)?;

        if index >= config.urls.len() {
            uefi::println!("Error: Index {} out of range (max: {})", index, config.urls.len() - 1);
            return Err(Error::NotFound);
        }

        let url = &config.urls[index];
        uefi::println!();
        uefi::println!("Booting image [{}]: {}", index, url);
        uefi::println!();

        // Download the image
        let image_data = crate::network::http::download(url)?;
        uefi::println!();
        uefi::println!("Download successful: {} bytes", image_data.len());

        // Verify SHA256 signature if present
        if index < config.signatures.len() && !config.signatures[index].is_empty() {
            let signature = &config.signatures[index];
            uefi::println!();
            match crate::network::verify::verify_signature(&image_data, signature) {
                Ok(_) => {
                    uefi::println!();
                }
                Err(e) => {
                    uefi::println!();
                    uefi::println!("SECURITY WARNING: Signature verification failed!");
                    uefi::println!("Refusing to boot unsigned/mismatched image.");
                    return Err(e);
                }
            }
        } else {
            uefi::println!();
            uefi::println!("WARNING: No signature configured for this image!");
            uefi::println!("Skipping verification (not recommended for production)");
        }

        // Chainload the verified image
        uefi::println!();
        crate::boot::chainload_image(&image_data)
    }

    fn exec_default(index: usize) -> Result<()> {
        let config = storage::get_config_mut().ok_or(Error::Unknown)?;

        if index >= config.urls.len() {
            uefi::println!("Error: Index {} out of range (max: {})", index, config.urls.len() - 1);
            return Err(Error::NotFound);
        }

        config.set_default(index)?;

        uefi::println!("Default image set to: [{}] {}", index, config.urls[index]);
        uefi::println!("Remember to run 'save' to persist changes to ESP");

        Ok(())
    }

    fn exec_save() -> Result<()> {
        let config = storage::get_config().ok_or(Error::Unknown)?;

        uefi::println!("Saving configuration to ESP...");

        match storage::save_config(config) {
            Ok(_) => {
                uefi::println!("Configuration saved successfully!");
                Ok(())
            }
            Err(e) => {
                uefi::println!("Error saving configuration: {}", e);
                Err(e)
            }
        }
    }

    fn exec_test_network() -> Result<()> {
        uefi::println!("Testing network connectivity...");
        uefi::println!();

        // Show network status
        crate::network::init::check_network_status()?;

        uefi::println!();

        // Test basic network detection
        crate::network::http::test_network()
    }

    fn exec_logs() -> Result<()> {
        let logs = crate::util::logger::get_logs();

        if logs.is_empty() {
            uefi::println!("No log entries.");
        } else {
            uefi::println!();
            uefi::println!("Log entries:");
            uefi::println!("============");
            for entry in logs.iter() {
                uefi::println!("[{:5}] {}", entry.level, entry.message);
            }
            uefi::println!();
        }

        Ok(())
    }

    fn exec_exit() -> Result<()> {
        uefi::println!("Exiting to firmware...");
        Err(Error::Unknown) // This will cause the REPL to exit
    }
}
