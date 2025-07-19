pub mod encryption;
/// Modern cryptographic operations module
///
/// This module replaces the legacy cppForSwig/EncryptionUtils with memory-safe
/// Rust implementations using modern, audited cryptographic libraries.
///
/// Key improvements over legacy implementation:
/// - Argon2id KDF replaces custom ROMIX implementation
/// - ChaCha20Poly1305 AEAD replaces AES with manual MAC
/// - Automatic memory zeroization via zeroize crate
/// - BIP-340 Schnorr signatures via secp256k1 crate
pub mod kdf;
pub mod random;
pub mod signatures;

/// Default parameters for Argon2id KDF
pub const DEFAULT_ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB
pub const DEFAULT_ARGON2_TIME_COST: u32 = 3; // 3 iterations
pub const DEFAULT_ARGON2_PARALLELISM: u32 = 4; // 4 threads

/// Salt size for key derivation functions
pub const SALT_SIZE: usize = 32;

/// Nonce size for ChaCha20Poly1305
pub const NONCE_SIZE: usize = 12;

/// Re-exports for convenience
pub use encryption::{
    decrypt_data, decrypt_with_password, encrypt_data, encrypt_with_password, EncryptedData,
    SecureKey,
};
pub use kdf::derive_key_from_password;
pub use random::{generate_random_bytes, generate_salt};
pub use signatures::{sign_ecdsa, sign_schnorr, verify_ecdsa, verify_schnorr};
