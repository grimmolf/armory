/// Cryptographically secure random number generation
/// 
/// This module provides secure random number generation to replace
/// the legacy random number generation in EncryptionUtils.cpp.
/// 
/// Key improvements:
/// - Uses OS-provided entropy sources
/// - Proper error handling for entropy exhaustion
/// - Cross-platform compatibility
/// - No dependency on external CSPRNGs

use crate::error::{CryptoError, CryptoResult};
use crate::crypto::SALT_SIZE;
use getrandom::getrandom;

/// Generate cryptographically secure random bytes
/// 
/// This function uses the operating system's entropy source to generate
/// high-quality random bytes suitable for cryptographic operations.
/// 
/// # Arguments
/// * `length` - Number of random bytes to generate
/// 
/// # Returns
/// Vector of random bytes
/// 
/// # Example
/// ```rust
/// use armory_rust::crypto::generate_random_bytes;
/// 
/// let random_data = generate_random_bytes(32)?;
/// assert_eq!(random_data.len(), 32);
/// ```
pub fn generate_random_bytes(length: usize) -> CryptoResult<Vec<u8>> {
    let mut buffer = vec![0u8; length];
    getrandom(&mut buffer)
        .map_err(|_e| CryptoError::RandomGeneration)?;
    Ok(buffer)
}

/// Generate a cryptographically secure salt
/// 
/// This creates a random salt suitable for key derivation functions.
/// The salt size is set to 32 bytes by default for strong security.
/// 
/// # Returns
/// 32-byte random salt
pub fn generate_salt() -> CryptoResult<Vec<u8>> {
    generate_random_bytes(SALT_SIZE)
}

/// Generate a random nonce for encryption
/// 
/// This creates a random nonce suitable for AEAD encryption.
/// 
/// # Arguments
/// * `size` - Size of the nonce in bytes (typically 12 for ChaCha20Poly1305)
/// 
/// # Returns
/// Random nonce bytes
pub fn generate_nonce(size: usize) -> CryptoResult<Vec<u8>> {
    generate_random_bytes(size)
}

/// Generate a secure random private key
/// 
/// This generates 32 random bytes suitable for use as a secp256k1 private key.
/// The bytes are validated to ensure they form a valid private key.
/// 
/// # Returns
/// 32 bytes of random data suitable for private key use
pub fn generate_private_key_bytes() -> CryptoResult<[u8; 32]> {
    use secp256k1::{SecretKey, Secp256k1};
    
    let _secp = Secp256k1::new();
    loop {
        let mut bytes = [0u8; 32];
        getrandom(&mut bytes)
            .map_err(|_| CryptoError::RandomGeneration)?;
            
        // Validate that the bytes form a valid private key
        if SecretKey::from_slice(&bytes).is_ok() {
            return Ok(bytes);
        }
        // If not valid, try again (very rare occurrence)
    }
}

/// Fill an existing buffer with random bytes
/// 
/// This is useful when you want to avoid allocations or need to
/// overwrite existing sensitive data with random values.
/// 
/// # Arguments
/// * `buffer` - Mutable buffer to fill with random bytes
/// 
/// # Example
/// ```rust
/// use armory_rust::crypto::fill_random;
/// 
/// let mut buffer = [0u8; 16];
/// fill_random(&mut buffer)?;
/// // buffer now contains 16 random bytes
/// ```
pub fn fill_random(buffer: &mut [u8]) -> CryptoResult<()> {
    getrandom(buffer)
        .map_err(|_| CryptoError::RandomGeneration)?;
    Ok(())
}

/// Generate a random u64 value
/// 
/// This is useful for generating random transaction nonces,
/// wallet IDs, or other numeric identifiers.
/// 
/// # Returns
/// Random 64-bit unsigned integer
pub fn generate_random_u64() -> CryptoResult<u64> {
    let mut bytes = [0u8; 8];
    fill_random(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Generate a random u32 value
/// 
/// This is useful for generating random derivation indices
/// or other 32-bit identifiers.
/// 
/// # Returns
/// Random 32-bit unsigned integer
pub fn generate_random_u32() -> CryptoResult<u32> {
    let mut bytes = [0u8; 4];
    fill_random(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Test if the random number generator is working
/// 
/// This function performs a basic sanity check to ensure
/// the random number generator is functioning correctly.
/// 
/// # Returns
/// True if RNG appears to be working, false otherwise
pub fn test_rng() -> bool {
    // Generate two sets of random bytes and ensure they're different
    let bytes1 = generate_random_bytes(32);
    let bytes2 = generate_random_bytes(32);
    
    match (bytes1, bytes2) {
        (Ok(b1), Ok(b2)) => {
            // Should be extremely unlikely to be equal
            b1 != b2 && !b1.iter().all(|&x| x == 0) && !b2.iter().all(|&x| x == 0)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_bytes_generation() {
        let bytes = generate_random_bytes(32).unwrap();
        assert_eq!(bytes.len(), 32);
        
        // Generate another set - should be different
        let bytes2 = generate_random_bytes(32).unwrap();
        assert_ne!(bytes, bytes2);
    }
    
    #[test]
    fn test_salt_generation() {
        let salt = generate_salt().unwrap();
        assert_eq!(salt.len(), SALT_SIZE);
        
        // Multiple salts should be different
        let salt2 = generate_salt().unwrap();
        assert_ne!(salt, salt2);
    }
    
    #[test]
    fn test_nonce_generation() {
        let nonce = generate_nonce(12).unwrap();
        assert_eq!(nonce.len(), 12);
        
        // Multiple nonces should be different
        let nonce2 = generate_nonce(12).unwrap();
        assert_ne!(nonce, nonce2);
    }
    
    #[test]
    fn test_private_key_bytes_generation() {
        let key_bytes1 = generate_private_key_bytes().unwrap();
        let key_bytes2 = generate_private_key_bytes().unwrap();
        
        // Should be different
        assert_ne!(key_bytes1, key_bytes2);
        
        // Should be valid private keys
        use secp256k1::SecretKey;
        assert!(SecretKey::from_slice(&key_bytes1).is_ok());
        assert!(SecretKey::from_slice(&key_bytes2).is_ok());
    }
    
    #[test]
    fn test_fill_random() {
        let mut buffer = [0u8; 16];
        fill_random(&mut buffer).unwrap();
        
        // Should not be all zeros
        assert!(buffer.iter().any(|&x| x != 0));
        
        let mut buffer2 = [0u8; 16];
        fill_random(&mut buffer2).unwrap();
        
        // Should be different
        assert_ne!(buffer, buffer2);
    }
    
    #[test]
    fn test_random_integers() {
        let val1 = generate_random_u64().unwrap();
        let val2 = generate_random_u64().unwrap();
        assert_ne!(val1, val2);
        
        let val3 = generate_random_u32().unwrap();
        let val4 = generate_random_u32().unwrap();
        assert_ne!(val3, val4);
    }
    
    #[test]
    fn test_rng_sanity_check() {
        assert!(test_rng());
    }
    
    #[test]
    fn test_zero_length_random() {
        let empty = generate_random_bytes(0).unwrap();
        assert_eq!(empty.len(), 0);
    }
    
    #[test]
    fn test_large_random_generation() {
        // Test generating larger amounts of random data
        let large_random = generate_random_bytes(1024).unwrap();
        assert_eq!(large_random.len(), 1024);
        
        // Should not be all the same value
        let first_byte = large_random[0];
        assert!(large_random.iter().any(|&x| x != first_byte));
    }
}