use crate::crypto::{generate_random_bytes, NONCE_SIZE};
/// Encryption module using ChaCha20Poly1305 AEAD
///
/// This module replaces the legacy AES implementation in EncryptionUtils.cpp
/// with modern AEAD (Authenticated Encryption with Associated Data) using
/// ChaCha20Poly1305, which provides both encryption and authentication.
///
/// Benefits over legacy AES implementation:
/// - Built-in authentication (no separate MAC needed)
/// - Resistant to timing attacks
/// - Fast on software (no AES-NI dependency)
/// - Automatic memory zeroization
use crate::error::{CryptoError, CryptoResult};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// Secure key wrapper that automatically zeroizes on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureKey {
    key: Key,
}

impl SecureKey {
    /// Create a new secure key from raw bytes
    pub fn from_bytes(key_bytes: &[u8]) -> CryptoResult<Self> {
        if key_bytes.len() != 32 {
            return Err(CryptoError::Encryption(
                "Key must be exactly 32 bytes".to_string(),
            ));
        }

        let mut key = Key::default();
        key.copy_from_slice(key_bytes);

        Ok(Self { key })
    }

    /// Generate a new random key
    pub fn generate() -> CryptoResult<Self> {
        let key_bytes = generate_random_bytes(32)?;
        Self::from_bytes(&key_bytes)
    }

    /// Derive a key from password using Argon2id
    pub fn derive_from_password(
        password: &str,
        salt: &[u8],
        memory_cost: u32,
        time_cost: u32,
    ) -> CryptoResult<Self> {
        use crate::crypto::kdf::{derive_key_from_password, KdfParams};

        let params = KdfParams::new(memory_cost, time_cost, 4, 32);
        let derived_key = derive_key_from_password(password, salt, &params)?;
        Self::from_bytes(&derived_key)
    }

    /// Get reference to the key (for internal use)
    fn key(&self) -> &Key {
        &self.key
    }
}

/// Encrypted data container that includes nonce and ciphertext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Random nonce used for encryption
    pub nonce: Vec<u8>,
    /// Encrypted and authenticated ciphertext
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Create new encrypted data container
    pub fn new(nonce: Vec<u8>, ciphertext: Vec<u8>) -> Self {
        Self { nonce, ciphertext }
    }

    /// Get the total size of encrypted data
    pub fn size(&self) -> usize {
        self.nonce.len() + self.ciphertext.len()
    }
}

/// Encrypt data using ChaCha20Poly1305 AEAD
///
/// This function replaces the legacy AES encryption with modern AEAD.
/// The nonce is automatically generated and included in the output.
///
/// # Arguments
/// * `key` - Secure encryption key
/// * `plaintext` - Data to encrypt
/// * `associated_data` - Optional associated data (authenticated but not encrypted)
///
/// # Returns
/// Encrypted data container with nonce and ciphertext
pub fn encrypt_data(
    key: &SecureKey,
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> CryptoResult<EncryptedData> {
    // Generate random nonce
    let nonce_bytes = generate_random_bytes(NONCE_SIZE)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new(key.key());

    // Encrypt with optional associated data
    let ciphertext = if let Some(aad) = associated_data {
        cipher.encrypt(
            nonce,
            chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad,
            },
        )
    } else {
        cipher.encrypt(nonce, plaintext)
    }
    .map_err(|e| CryptoError::Encryption(format!("Encryption failed: {e}")))?;

    Ok(EncryptedData::new(nonce_bytes, ciphertext))
}

/// Decrypt data using ChaCha20Poly1305 AEAD
///
/// This function replaces the legacy AES decryption with modern AEAD.
/// The authentication tag is automatically verified.
///
/// # Arguments
/// * `key` - Secure encryption key (must match encryption key)
/// * `encrypted_data` - Encrypted data container
/// * `associated_data` - Optional associated data (must match encryption)
///
/// # Returns
/// Decrypted plaintext
pub fn decrypt_data(
    key: &SecureKey,
    encrypted_data: &EncryptedData,
    associated_data: Option<&[u8]>,
) -> CryptoResult<Vec<u8>> {
    // Validate nonce size
    if encrypted_data.nonce.len() != NONCE_SIZE {
        return Err(CryptoError::Decryption(format!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            encrypted_data.nonce.len()
        )));
    }

    let nonce = Nonce::from_slice(&encrypted_data.nonce);

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new(key.key());

    // Decrypt and verify authentication
    let plaintext = if let Some(aad) = associated_data {
        cipher.decrypt(
            nonce,
            chacha20poly1305::aead::Payload {
                msg: &encrypted_data.ciphertext,
                aad,
            },
        )
    } else {
        cipher.decrypt(nonce, encrypted_data.ciphertext.as_slice())
    }
    .map_err(|e| CryptoError::Decryption(format!("Decryption failed: {e}")))?;

    Ok(plaintext)
}

/// Encrypt data with password using Argon2id + ChaCha20Poly1305
///
/// This is a convenience function that combines key derivation and encryption.
/// Suitable for wallet file encryption and other password-protected data.
pub fn encrypt_with_password(
    password: &str,
    salt: &[u8],
    plaintext: &[u8],
    memory_cost: u32,
    time_cost: u32,
) -> CryptoResult<EncryptedData> {
    let key = SecureKey::derive_from_password(password, salt, memory_cost, time_cost)?;
    encrypt_data(&key, plaintext, None)
}

/// Decrypt data with password using Argon2id + ChaCha20Poly1305
///
/// This is a convenience function that combines key derivation and decryption.
/// Must use the same parameters as encryption.
pub fn decrypt_with_password(
    password: &str,
    salt: &[u8],
    encrypted_data: &EncryptedData,
    memory_cost: u32,
    time_cost: u32,
) -> CryptoResult<Vec<u8>> {
    let key = SecureKey::derive_from_password(password, salt, memory_cost, time_cost)?;
    decrypt_data(&key, encrypted_data, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = SecureKey::generate().unwrap();
        // Key should be created successfully
        assert_eq!(key.key().len(), 32);
    }

    #[test]
    fn test_key_from_bytes() {
        let key_bytes = [1u8; 32];
        let key = SecureKey::from_bytes(&key_bytes).unwrap();
        assert_eq!(key.key().as_slice(), &key_bytes);

        // Wrong size should fail
        let wrong_size = [1u8; 16];
        assert!(SecureKey::from_bytes(&wrong_size).is_err());
    }

    #[test]
    fn test_encryption_decryption() {
        let key = SecureKey::generate().unwrap();
        let plaintext = b"Hello, Bitcoin!";

        let encrypted = encrypt_data(&key, plaintext, None).unwrap();
        let decrypted = decrypt_data(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
        assert_eq!(encrypted.nonce.len(), NONCE_SIZE);
        assert_ne!(encrypted.ciphertext, plaintext);
    }

    #[test]
    fn test_encryption_with_associated_data() {
        let key = SecureKey::generate().unwrap();
        let plaintext = b"Secret message";
        let aad = b"public metadata";

        let encrypted = encrypt_data(&key, plaintext, Some(aad)).unwrap();
        let decrypted = decrypt_data(&key, &encrypted, Some(aad)).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());

        // Decryption with wrong AAD should fail
        let wrong_aad = b"wrong metadata";
        assert!(decrypt_data(&key, &encrypted, Some(wrong_aad)).is_err());
    }

    #[test]
    fn test_password_based_encryption() {
        let password = "strong_password_2025";
        let salt = b"random_salt_for_test_must_be_long";
        let plaintext = b"Wallet seed data";

        let encrypted = encrypt_with_password(password, salt, plaintext, 1024, 1).unwrap();

        let decrypted = decrypt_with_password(password, salt, &encrypted, 1024, 1).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());

        // Wrong password should fail
        assert!(decrypt_with_password("wrong_password", salt, &encrypted, 1024, 1).is_err());
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = SecureKey::generate().unwrap();
        let key2 = SecureKey::generate().unwrap();
        let plaintext = b"Same plaintext";

        let encrypted1 = encrypt_data(&key1, plaintext, None).unwrap();
        let encrypted2 = encrypt_data(&key2, plaintext, None).unwrap();

        // Different keys should produce different ciphertext
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Decryption with wrong key should fail
        assert!(decrypt_data(&key1, &encrypted2, None).is_err());
    }

    #[test]
    fn test_tampering_detection() {
        let key = SecureKey::generate().unwrap();
        let plaintext = b"Important data";

        let mut encrypted = encrypt_data(&key, plaintext, None).unwrap();

        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 1;

        // Decryption should fail due to authentication
        assert!(decrypt_data(&key, &encrypted, None).is_err());
    }
}
