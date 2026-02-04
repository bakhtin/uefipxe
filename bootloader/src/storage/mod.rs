pub mod config;
pub mod file;

use crate::util::{Error, Result};

pub use config::Config;

/// Load configuration from ESP
pub fn load_config() -> Result<Config> {
    match file::read_file(file::CONFIG_PATH) {
        Ok(data) => {
            // Convert bytes to string
            let content = core::str::from_utf8(&data).map_err(|_| Error::Parse)?;

            // Parse configuration
            Config::parse(content)
        }
        Err(Error::NotFound) => {
            // Config file doesn't exist, return empty config
            uefi::println!("Config file not found, using empty configuration");
            Ok(Config::new())
        }
        Err(e) => Err(e),
    }
}

/// Save configuration to ESP
pub fn save_config(config: &Config) -> Result<()> {
    // Serialize configuration
    let content = config.serialize()?;

    // Write to file
    file::write_file(file::CONFIG_PATH, content.as_bytes())?;

    Ok(())
}

/// Global configuration state
static mut GLOBAL_CONFIG: Option<Config> = None;

/// Initialize global configuration
pub fn init_config(config: Config) {
    unsafe {
        GLOBAL_CONFIG = Some(config);
    }
}

/// Get a reference to the global configuration
pub fn get_config() -> Option<&'static Config> {
    unsafe { GLOBAL_CONFIG.as_ref() }
}

/// Get a mutable reference to the global configuration
pub fn get_config_mut() -> Option<&'static mut Config> {
    unsafe { GLOBAL_CONFIG.as_mut() }
}
