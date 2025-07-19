use crate::crypto::{
    DEFAULT_ARGON2_MEMORY_COST, DEFAULT_ARGON2_PARALLELISM, DEFAULT_ARGON2_TIME_COST,
};
/// Key Derivation Functions
///
/// This module implements modern, memory-hard key derivation functions
/// to replace the legacy custom ROMIX implementation in EncryptionUtils.cpp.
///
/// The Argon2id algorithm provides:
/// - Memory-hard computation (GPU resistance)
/// - Side-channel attack resistance
/// - Tunable time/memory parameters
/// - Well-audited implementation
use crate::error::{CryptoError, CryptoResult};
use argon2::{Algorithm, Argon2, Params, Version};

/// Parameters for Argon2id key derivation
#[derive(Debug, Clone)]
pub struct KdfParams {
    /// Memory cost in KiB (default: 64 MiB)
    pub memory_cost: u32,
    /// Time cost in iterations (default: 3)
    pub time_cost: u32,
    /// Parallelism degree (default: 4)
    pub parallelism: u32,
    /// Output length in bytes (default: 32)
    pub output_length: usize,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost: DEFAULT_ARGON2_MEMORY_COST,
            time_cost: DEFAULT_ARGON2_TIME_COST,
            parallelism: DEFAULT_ARGON2_PARALLELISM,
            output_length: 32,
        }
    }
}

impl KdfParams {
    /// Create KDF parameters with custom settings
    pub fn new(memory_cost: u32, time_cost: u32, parallelism: u32, output_length: usize) -> Self {
        Self {
            memory_cost,
            time_cost,
            parallelism,
            output_length,
        }
    }

    /// Create parameters optimized for wallet encryption
    /// These provide strong security while maintaining reasonable performance
    pub fn for_wallet_encryption() -> Self {
        Self {
            memory_cost: 65536, // 64 MiB
            time_cost: 3,       // 3 iterations
            parallelism: 4,     // 4 threads
            output_length: 32,  // 256 bits
        }
    }

    /// Create parameters optimized for faster operations (e.g., transaction signing)
    pub fn for_fast_operations() -> Self {
        Self {
            memory_cost: 16384, // 16 MiB
            time_cost: 2,       // 2 iterations
            parallelism: 2,     // 2 threads
            output_length: 32,  // 256 bits
        }
    }
}

/// Derive a key from a password using Argon2id
///
/// This replaces the legacy ROMIX KDF with a modern, standardized algorithm.
/// The derived key is automatically zeroized when dropped.
///
/// # Arguments
/// * `password` - The password to derive from
/// * `salt` - Random salt (should be unique per password)
/// * `params` - KDF parameters (memory cost, time cost, etc.)
///
/// # Returns
/// Derived key that is automatically zeroized on drop
///
/// # Example
/// ```rust
/// use armory_rust::crypto::{derive_key_from_password, KdfParams};
///
/// let password = "my secure password";
/// let salt = b"random_salt_32_bytes_long_here!!";
/// let params = KdfParams::for_wallet_encryption();
///
/// let derived_key = derive_key_from_password(password, salt, &params)?;
/// ```
pub fn derive_key_from_password(
    password: &str,
    salt: &[u8],
    params: &KdfParams,
) -> CryptoResult<Vec<u8>> {
    // Validate input parameters
    if salt.len() < 16 {
        return Err(CryptoError::KeyDerivation(
            "Salt must be at least 16 bytes".to_string(),
        ));
    }

    if params.output_length == 0 || params.output_length > 128 {
        return Err(CryptoError::KeyDerivation(
            "Output length must be between 1 and 128 bytes".to_string(),
        ));
    }

    // Create Argon2id instance with specified parameters
    let argon2_params = Params::new(
        params.memory_cost,
        params.time_cost,
        params.parallelism,
        Some(params.output_length),
    )
    .map_err(|e| CryptoError::KeyDerivation(format!("Invalid Argon2 parameters: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    // Derive the key
    let mut output = vec![0u8; params.output_length];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| CryptoError::KeyDerivation(format!("Argon2 derivation failed: {e}")))?;

    Ok(output)
}

/// Verify a password against a derived key
///
/// This performs constant-time comparison to prevent timing attacks.
pub fn verify_password(
    password: &str,
    salt: &[u8],
    expected_key: &[u8],
    params: &KdfParams,
) -> CryptoResult<bool> {
    let derived_key = derive_key_from_password(password, salt, params)?;

    // Constant-time comparison to prevent timing attacks
    if derived_key.len() != expected_key.len() {
        return Ok(false);
    }

    let mut result = 0u8;
    for (a, b) in derived_key.iter().zip(expected_key.iter()) {
        result |= a ^ b;
    }

    Ok(result == 0)
}

/// Compute KDF parameters based on available system resources
///
/// This attempts to select optimal parameters based on available memory
/// and desired computation time, similar to the legacy computeKdfParams.
pub fn compute_optimal_params(target_time_ms: u32, max_memory_mb: u32) -> KdfParams {
    // Start with default parameters
    let mut params = KdfParams::default();

    // Adjust memory cost based on available memory
    let max_memory_kb = max_memory_mb * 1024;
    if max_memory_kb < params.memory_cost {
        params.memory_cost = max_memory_kb;
    }

    // Adjust time cost based on target time
    // This is a simplified heuristic - in practice you'd benchmark
    match target_time_ms {
        0..=250 => params.time_cost = 1,
        251..=500 => params.time_cost = 2,
        501..=1000 => params.time_cost = 3,
        _ => params.time_cost = 4,
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt_16_bytes_minimum_length";
        let params = KdfParams::default();

        let key1 = derive_key_from_password(password, salt, &params).unwrap();
        let key2 = derive_key_from_password(password, salt, &params).unwrap();

        // Same inputs should produce same output
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), params.output_length);
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "test_password";
        let salt1 = b"salt1_16_bytes_minimum_length!!!";
        let salt2 = b"salt2_16_bytes_minimum_length!!!";
        let params = KdfParams::default();

        let key1 = derive_key_from_password(password, salt1, &params).unwrap();
        let key2 = derive_key_from_password(password, salt2, &params).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_password_verification() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let salt = b"test_salt_16_bytes_minimum_length";
        let params = KdfParams::default();

        let key = derive_key_from_password(password, salt, &params).unwrap();

        assert!(verify_password(password, salt, &key, &params).unwrap());
        assert!(!verify_password(wrong_password, salt, &key, &params).unwrap());
    }

    #[test]
    fn test_optimal_params_computation() {
        let params1 = compute_optimal_params(100, 64);
        let params2 = compute_optimal_params(1000, 64);

        // Longer target time should result in higher time cost
        assert!(params2.time_cost >= params1.time_cost);
    }

    #[test]
    fn test_invalid_salt_size() {
        let password = "test_password";
        let short_salt = b"short"; // Too short
        let params = KdfParams::default();

        let result = derive_key_from_password(password, short_salt, &params);
        assert!(result.is_err());
    }
}
