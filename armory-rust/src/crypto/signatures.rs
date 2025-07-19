/// Digital signature operations module
///
/// This module implements both ECDSA and BIP-340 Schnorr signatures,
/// replacing the legacy ECDSA-only implementation in EncryptionUtils.cpp.
///
/// Key improvements:
/// - BIP-340 Schnorr signatures for Taproot support
/// - Deterministic nonce generation (RFC 6979)
/// - Memory-safe key handling
/// - Consistent error handling
use crate::error::{CryptoError, CryptoResult};
use secp256k1::{
    ecdsa::Signature, schnorr, All, Keypair, Message, PublicKey, Secp256k1, SecretKey,
    XOnlyPublicKey,
};
use std::fmt;
use zeroize::Zeroize;

/// Secp256k1 context for cryptographic operations
pub struct SignatureContext {
    secp: Secp256k1<All>,
}

impl SignatureContext {
    /// Create a new signature context
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
        }
    }

    /// Get reference to secp256k1 context
    pub fn secp(&self) -> &Secp256k1<All> {
        &self.secp
    }
}

impl Default for SignatureContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A secure wrapper around secp256k1 private key
pub struct PrivateKey {
    secret_key: SecretKey,
    context: SignatureContext,
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        // Manually zeroize the secret key memory
        // SecretKey doesn't implement Zeroize, so we work around it
        use std::ptr;
        unsafe {
            ptr::write_volatile(&mut self.secret_key as *mut _ as *mut [u8; 32], [0u8; 32]);
        }
    }
}

impl PrivateKey {
    /// Create private key from 32 bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidPrivateKey);
        }

        let secret_key =
            SecretKey::from_slice(bytes).map_err(|_| CryptoError::InvalidPrivateKey)?;

        Ok(Self {
            secret_key,
            context: SignatureContext::new(),
        })
    }

    /// Generate a new random private key
    pub fn generate() -> CryptoResult<Self> {
        use crate::crypto::random::generate_random_bytes;

        let mut bytes = generate_random_bytes(32)?;
        let result = Self::from_bytes(&bytes);
        bytes.zeroize(); // Clear the random bytes
        result
    }

    /// Get the corresponding public key
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_secret_key(self.context.secp(), &self.secret_key)
    }

    /// Get the corresponding x-only public key for Taproot
    pub fn x_only_public_key(&self) -> XOnlyPublicKey {
        let keypair = Keypair::from_secret_key(self.context.secp(), &self.secret_key);
        XOnlyPublicKey::from_keypair(&keypair).0
    }

    /// Convert to keypair for Schnorr operations
    pub fn to_keypair(&self) -> Keypair {
        Keypair::from_secret_key(self.context.secp(), &self.secret_key)
    }

    /// Get reference to the secret key (internal use)
    fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    /// Get reference to the context
    fn context(&self) -> &SignatureContext {
        &self.context
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKey")
            .field("public_key", &self.public_key())
            .finish()
    }
}

/// BIP-340 Schnorr signature operations
pub mod schnorr_ops {
    use super::*;

    /// Sign a message using BIP-340 Schnorr signatures
    ///
    /// This implements the BIP-340 specification for Schnorr signatures
    /// used in Bitcoin Taproot transactions.
    ///
    /// # Arguments
    /// * `private_key` - The private key to sign with
    /// * `message` - 32-byte message hash to sign
    ///
    /// # Returns
    /// 64-byte Schnorr signature
    pub fn sign(private_key: &PrivateKey, message: &[u8]) -> CryptoResult<[u8; 64]> {
        if message.len() != 32 {
            return Err(CryptoError::SignatureVerification);
        }

        let msg =
            Message::from_digest_slice(message).map_err(|_| CryptoError::SignatureVerification)?;

        let keypair = private_key.to_keypair();
        // Use zero auxiliary randomness for deterministic signatures (BIP-340 compliant)
        let aux_rand = [0u8; 32];
        let signature = private_key
            .context()
            .secp()
            .sign_schnorr_with_aux_rand(&msg, &keypair, &aux_rand);

        Ok(*signature.as_ref())
    }

    /// Verify a BIP-340 Schnorr signature
    ///
    /// # Arguments
    /// * `public_key` - The x-only public key that signed the message
    /// * `message` - 32-byte message hash that was signed
    /// * `signature` - 64-byte Schnorr signature to verify
    ///
    /// # Returns
    /// True if signature is valid, false otherwise
    pub fn verify(
        public_key: &XOnlyPublicKey,
        message: &[u8],
        signature: &[u8; 64],
    ) -> CryptoResult<bool> {
        if message.len() != 32 {
            return Err(CryptoError::SignatureVerification);
        }

        let secp = Secp256k1::verification_only();
        let msg =
            Message::from_digest_slice(message).map_err(|_| CryptoError::SignatureVerification)?;

        let sig = schnorr::Signature::from_slice(signature)
            .map_err(|_| CryptoError::SignatureVerification)?;

        match secp.verify_schnorr(&sig, &msg, public_key) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// ECDSA signature operations for compatibility
pub mod ecdsa_ops {
    use super::*;

    /// Sign a message using ECDSA
    ///
    /// This provides compatibility with legacy Bitcoin transactions
    /// and pre-Taproot address types.
    ///
    /// # Arguments
    /// * `private_key` - The private key to sign with
    /// * `message` - 32-byte message hash to sign
    ///
    /// # Returns
    /// DER-encoded ECDSA signature
    pub fn sign(private_key: &PrivateKey, message: &[u8]) -> CryptoResult<Vec<u8>> {
        if message.len() != 32 {
            return Err(CryptoError::SignatureVerification);
        }

        let msg =
            Message::from_digest_slice(message).map_err(|_| CryptoError::SignatureVerification)?;

        let signature = private_key
            .context()
            .secp()
            .sign_ecdsa(&msg, private_key.secret_key());
        Ok(signature.serialize_der().to_vec())
    }

    /// Verify an ECDSA signature
    ///
    /// # Arguments
    /// * `public_key` - The public key that signed the message
    /// * `message` - 32-byte message hash that was signed
    /// * `signature` - DER-encoded ECDSA signature to verify
    ///
    /// # Returns
    /// True if signature is valid, false otherwise
    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        if message.len() != 32 {
            return Err(CryptoError::SignatureVerification);
        }

        let secp = Secp256k1::verification_only();
        let msg =
            Message::from_digest_slice(message).map_err(|_| CryptoError::SignatureVerification)?;

        let sig = Signature::from_der(signature).map_err(|_| CryptoError::SignatureVerification)?;

        match secp.verify_ecdsa(&msg, &sig, public_key) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// High-level signature functions for convenience
///
/// These functions automatically choose the appropriate signature type
/// based on the context and provide a simplified interface.
/// Sign a message using BIP-340 Schnorr signatures
///
/// This is the recommended signature method for new applications
/// as it provides better efficiency and privacy properties.
pub fn sign_schnorr(message: &[u8], private_key: &PrivateKey) -> CryptoResult<[u8; 64]> {
    schnorr_ops::sign(private_key, message)
}

/// Verify a BIP-340 Schnorr signature
pub fn verify_schnorr(
    message: &[u8],
    signature: &[u8; 64],
    public_key: &XOnlyPublicKey,
) -> CryptoResult<bool> {
    schnorr_ops::verify(public_key, message, signature)
}

/// Sign a message using ECDSA
///
/// This provides compatibility with legacy Bitcoin transactions.
pub fn sign_ecdsa(message: &[u8], private_key: &PrivateKey) -> CryptoResult<Vec<u8>> {
    ecdsa_ops::sign(private_key, message)
}

/// Verify an ECDSA signature
pub fn verify_ecdsa(
    message: &[u8],
    signature: &[u8],
    public_key: &PublicKey,
) -> CryptoResult<bool> {
    ecdsa_ops::verify(public_key, message, signature)
}

/// Generate a message hash from arbitrary data
///
/// This creates a 32-byte SHA256 hash suitable for signing.
pub fn hash_message(data: &[u8]) -> [u8; 32] {
    use bitcoin::hashes::{sha256, Hash};
    sha256::Hash::hash(data).to_byte_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_key_generation() {
        let key1 = PrivateKey::generate().unwrap();
        let key2 = PrivateKey::generate().unwrap();

        // Keys should be different
        assert_ne!(key1.public_key(), key2.public_key());
    }

    #[test]
    fn test_private_key_from_bytes() {
        let bytes = [1u8; 32];
        let key = PrivateKey::from_bytes(&bytes).unwrap();

        // Should create valid public key
        let _public_key = key.public_key(); // If this doesn't panic, the key is valid

        // Invalid length should fail
        let invalid_bytes = [1u8; 31];
        assert!(PrivateKey::from_bytes(&invalid_bytes).is_err());
    }

    #[test]
    fn test_schnorr_sign_verify() {
        let private_key = PrivateKey::generate().unwrap();
        let x_only_pubkey = private_key.x_only_public_key();
        let message = hash_message(b"Hello Bitcoin 2025!");

        // Sign the message
        let signature = sign_schnorr(&message, &private_key).unwrap();

        // Verify the signature
        assert!(verify_schnorr(&message, &signature, &x_only_pubkey).unwrap());

        // Wrong message should fail verification
        let wrong_message = hash_message(b"Wrong message");
        assert!(!verify_schnorr(&wrong_message, &signature, &x_only_pubkey).unwrap());
    }

    #[test]
    fn test_ecdsa_sign_verify() {
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key();
        let message = hash_message(b"ECDSA test message");

        // Sign the message
        let signature = sign_ecdsa(&message, &private_key).unwrap();

        // Verify the signature
        assert!(verify_ecdsa(&message, &signature, &public_key).unwrap());

        // Wrong message should fail verification
        let wrong_message = hash_message(b"Different message");
        assert!(!verify_ecdsa(&wrong_message, &signature, &public_key).unwrap());
    }

    #[test]
    fn test_deterministic_signatures() {
        let key_bytes = [42u8; 32];
        let private_key1 = PrivateKey::from_bytes(&key_bytes).unwrap();
        let private_key2 = PrivateKey::from_bytes(&key_bytes).unwrap();
        let message = hash_message(b"Deterministic test");

        let sig1 = sign_schnorr(&message, &private_key1).unwrap();
        let sig2 = sign_schnorr(&message, &private_key2).unwrap();

        // Same key and message should produce same signature (deterministic)
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_invalid_message_length() {
        let private_key = PrivateKey::generate().unwrap();
        let short_message = b"short"; // Not 32 bytes

        // Should fail with invalid message length
        assert!(sign_schnorr(short_message, &private_key).is_err());
        assert!(sign_ecdsa(short_message, &private_key).is_err());
    }

    #[test]
    fn test_cross_compatibility() {
        // Test that we can convert between different representations
        let private_key = PrivateKey::generate().unwrap();
        let public_key = private_key.public_key();
        let x_only_pubkey = private_key.x_only_public_key();

        // X-only pubkey should be derived from the same key
        let derived_x_only = XOnlyPublicKey::from(public_key);
        assert_eq!(x_only_pubkey, derived_x_only);
    }
}
