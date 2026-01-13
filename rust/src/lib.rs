pub mod core;

pub use core::turb1600_hash;

/// Convenience: hash a string to hex
pub fn hash_hex(data: &str) -> String {
    let digest = turb1600_hash(data.as_bytes());
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hash() {
        let msg = b"hello world";
        let digest = turb1600_hash(msg);
        assert_eq!(digest.len(), 128);
    }

    #[test]
    fn test_hash_hex() {
        let hex = hash_hex("test");
        assert_eq!(hex.len(), 256); // 128 bytes -> 256 hex chars
    }
}
