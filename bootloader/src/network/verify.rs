use crate::util::{Error, Result};
use alloc::format;
use alloc::string::String;
use sha2::{Digest, Sha256};
use uefi::println;

/// Compute SHA256 hash of data and return as lowercase hex string
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();

    // Process in chunks for better memory efficiency with large files
    for chunk in data.chunks(8192) {
        hasher.update(chunk);
    }

    let result = hasher.finalize();

    // Convert to lowercase hex string
    format!("{:x}", result)
}

/// Verify that downloaded data matches expected SHA256 signature
pub fn verify_signature(data: &[u8], expected_signature: &str) -> Result<()> {
    println!("  Verifying signature...");

    // Compute actual hash
    let actual_hash = compute_sha256(data);

    println!("  Expected: {}", expected_signature);
    println!("  Actual:   {}", actual_hash);

    // Compare signatures (case-insensitive)
    if actual_hash.eq_ignore_ascii_case(expected_signature) {
        println!("  ✓ Signature verification passed");
        Ok(())
    } else {
        println!("  ✗ Signature verification FAILED");
        Err(Error::Io) // Use Io error for signature mismatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = compute_sha256(&[]);
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        let hash = compute_sha256(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_verify_signature_success() {
        let data = b"hello";
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert!(verify_signature(data, expected).is_ok());
    }

    #[test]
    fn test_verify_signature_failure() {
        let data = b"hello";
        let expected = "invalid_hash";
        assert!(verify_signature(data, expected).is_err());
    }
}
