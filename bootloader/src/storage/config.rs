use crate::util::{Error, Result};
use heapless::{String, Vec};
use core::fmt::Write;

/// Maximum number of image URLs that can be stored
pub const MAX_URLS: usize = 16;

/// Maximum length of a URL
pub const MAX_URL_LEN: usize = 256;

/// Maximum length of a signature (hex-encoded SHA256 = 64 chars)
pub const MAX_SIGNATURE_LEN: usize = 128;

/// Configuration for the bootloader
#[derive(Debug, Clone)]
pub struct Config {
    /// List of image URLs
    pub urls: Vec<String<MAX_URL_LEN>, MAX_URLS>,
    /// List of image signatures (SHA256 hex, empty string = no verification)
    pub signatures: Vec<String<MAX_SIGNATURE_LEN>, MAX_URLS>,
    /// Default image index (0-based)
    pub default_index: Option<usize>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Config {
            urls: Vec::new(),
            signatures: Vec::new(),
            default_index: None,
        }
    }

    /// Add a URL to the configuration (without signature)
    pub fn add_url(&mut self, url: &str) -> Result<()> {
        self.add_url_with_signature(url, "")
    }

    /// Add a URL with signature to the configuration
    pub fn add_url_with_signature(&mut self, url: &str, signature: &str) -> Result<()> {
        if self.urls.is_full() {
            return Err(Error::OutOfMemory);
        }

        let mut url_string = String::new();
        url_string.push_str(url).map_err(|_| Error::BufferTooSmall)?;

        let mut sig_string = String::new();
        sig_string.push_str(signature).map_err(|_| Error::BufferTooSmall)?;

        self.urls.push(url_string).map_err(|_| Error::OutOfMemory)?;
        self.signatures.push(sig_string).map_err(|_| Error::OutOfMemory)?;
        Ok(())
    }

    /// Remove a URL at the specified index
    pub fn remove_url(&mut self, index: usize) -> Result<()> {
        if index >= self.urls.len() {
            return Err(Error::NotFound);
        }

        self.urls.remove(index);
        self.signatures.remove(index);

        // Adjust default index if necessary
        if let Some(default) = self.default_index {
            if default == index {
                self.default_index = None;
            } else if default > index {
                self.default_index = Some(default - 1);
            }
        }

        Ok(())
    }

    /// Set the default image index
    pub fn set_default(&mut self, index: usize) -> Result<()> {
        if index >= self.urls.len() {
            return Err(Error::NotFound);
        }

        self.default_index = Some(index);
        Ok(())
    }

    /// Parse configuration from text content
    pub fn parse(content: &str) -> Result<Self> {
        let mut config = Config::new();
        let mut last_url_index = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "default" => {
                        let index = value.parse::<usize>().map_err(|_| Error::Parse)?;
                        config.default_index = Some(index);
                    }
                    "url" => {
                        config.add_url(value)?;
                        last_url_index = Some(config.urls.len() - 1);
                    }
                    "signature" | "sha256" => {
                        // Signature follows the last URL
                        if let Some(idx) = last_url_index {
                            if idx < config.signatures.len() {
                                config.signatures[idx].clear();
                                config.signatures[idx].push_str(value).map_err(|_| Error::BufferTooSmall)?;
                            }
                        }
                    }
                    _ => {
                        // Unknown key, skip
                    }
                }
            }
        }

        Ok(config)
    }

    /// Serialize configuration to text format
    pub fn serialize(&self) -> Result<String<4096>> {
        let mut output = String::new();

        // Write header
        writeln!(output, "# UEFI PXE Bootloader Configuration").map_err(|_| Error::BufferTooSmall)?;
        writeln!(output, "# Lines starting with # are comments").map_err(|_| Error::BufferTooSmall)?;
        writeln!(output).map_err(|_| Error::BufferTooSmall)?;

        // Write default index
        if let Some(default) = self.default_index {
            writeln!(output, "default={}", default).map_err(|_| Error::BufferTooSmall)?;
            writeln!(output).map_err(|_| Error::BufferTooSmall)?;
        }

        // Write URLs with signatures
        writeln!(output, "# Image URLs with optional SHA256 signatures").map_err(|_| Error::BufferTooSmall)?;
        for (i, url) in self.urls.iter().enumerate() {
            writeln!(output, "url={}", url).map_err(|_| Error::BufferTooSmall)?;
            if i < self.signatures.len() && !self.signatures[i].is_empty() {
                writeln!(output, "sha256={}", self.signatures[i]).map_err(|_| Error::BufferTooSmall)?;
            }
        }

        Ok(output)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = Config::new();
        assert_eq!(config.urls.len(), 0);
        assert_eq!(config.default_index, None);
    }

    #[test]
    fn test_add_url() {
        let mut config = Config::new();
        assert!(config.add_url("https://example.com/image.efi").is_ok());
        assert_eq!(config.urls.len(), 1);
    }

    #[test]
    fn test_remove_url() {
        let mut config = Config::new();
        config.add_url("https://example.com/image1.efi").unwrap();
        config.add_url("https://example.com/image2.efi").unwrap();

        assert!(config.remove_url(0).is_ok());
        assert_eq!(config.urls.len(), 1);
        assert_eq!(config.urls[0].as_str(), "https://example.com/image2.efi");
    }

    #[test]
    fn test_set_default() {
        let mut config = Config::new();
        config.add_url("https://example.com/image.efi").unwrap();

        assert!(config.set_default(0).is_ok());
        assert_eq!(config.default_index, Some(0));
    }

    #[test]
    fn test_parse_empty() {
        let content = "";
        let config = Config::parse(content).unwrap();
        assert_eq!(config.urls.len(), 0);
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# This is a comment
default=0
# Another comment
url=https://example.com/image1.efi
url=https://example.com/image2.efi
"#;
        let config = Config::parse(content).unwrap();
        assert_eq!(config.urls.len(), 2);
        assert_eq!(config.default_index, Some(0));
    }

    #[test]
    fn test_serialize() {
        let mut config = Config::new();
        config.add_url("https://example.com/image.efi").unwrap();
        config.set_default(0).unwrap();

        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("default=0"));
        assert!(serialized.contains("url=https://example.com/image.efi"));
    }
}
