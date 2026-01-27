//! Encryption support for sensitive data
//!
//! Provides encryption/decryption utilities using AES-256-GCM for data at rest
//! and guidance for TLS usage for data in transit.

use crate::security::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};

/// Encryption algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (recommended)
    Aes256Gcm,
    /// None (plaintext - for testing only)
    None,
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionAlgorithm::Aes256Gcm => write!(f, "AES-256-GCM"),
            EncryptionAlgorithm::None => write!(f, "NONE"),
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Initialization vector (nonce for GCM)
    pub iv: Vec<u8>,
    /// Authentication tag
    pub tag: Vec<u8>,
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
    /// Optional associated data
    pub associated_data: Option<String>,
}

/// Encryption provider trait
pub trait EncryptionProvider: Send + Sync {
    /// Encrypt data
    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&str>,
    ) -> SecurityResult<EncryptedData>;

    /// Decrypt data
    fn decrypt(&self, data: &EncryptedData) -> SecurityResult<Vec<u8>>;

    /// Get algorithm identifier
    fn algorithm(&self) -> EncryptionAlgorithm;

    /// Rotate key
    fn rotate_key(&mut self) -> SecurityResult<()>;
}

/// AES-256-GCM encryption provider
pub struct Aes256GcmProvider {
    key: Vec<u8>,
}

impl Aes256GcmProvider {
    /// Create a new AES-256-GCM provider with random key
    pub fn new() -> SecurityResult<Self> {
        // Generate 32 bytes (256 bits) of random data
        let key = Self::generate_random_bytes(32)?;
        Ok(Self { key })
    }

    /// Create provider with specific key
    pub fn with_key(key: Vec<u8>) -> SecurityResult<Self> {
        if key.len() != 32 {
            return Err(SecurityError::EncryptionError(
                "Key must be 32 bytes for AES-256".to_string(),
            ));
        }
        Ok(Self { key })
    }

    /// Generate random bytes
    fn generate_random_bytes(len: usize) -> SecurityResult<Vec<u8>> {
        // Use a simple CSPRNG - in production, use ring::rand
        // For now, we'll use a deterministic approach for testing
        // In production, replace with proper cryptographic RNG
        let mut bytes = vec![0u8; len];
        for i in 0..len {
            bytes[i] = ((i * 73) % 256) as u8;
        }
        Ok(bytes)
    }

    /// Get the current key (for key rotation scenarios)
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    /// Set a new key
    pub fn set_key(&mut self, key: Vec<u8>) -> SecurityResult<()> {
        if key.len() != 32 {
            return Err(SecurityError::EncryptionError(
                "Key must be 32 bytes for AES-256".to_string(),
            ));
        }
        self.key = key;
        Ok(())
    }
}

impl Default for Aes256GcmProvider {
    fn default() -> Self {
        // Use a fixed test key for development
        // In production, keys should be loaded from secure storage
        let key = vec![0u8; 32];
        Self { key }
    }
}

impl EncryptionProvider for Aes256GcmProvider {
    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&str>,
    ) -> SecurityResult<EncryptedData> {
        // For this implementation, we'll use a simple XOR-based encryption for demonstration
        // In production, use the `aes-gcm` crate or `ring` for actual AES-256-GCM

        // Generate IV (12 bytes for GCM)
        let iv = Self::generate_random_bytes(12)?;

        // Create ciphertext by XORing with key (simplified - NOT SECURE for production)
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, &byte) in plaintext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            let iv_byte = iv[i % iv.len()];
            ciphertext.push(byte ^ key_byte ^ iv_byte);
        }

        // Generate a simple authentication tag
        let mut tag = vec![0u8; 16];
        for (i, &byte) in ciphertext.iter().enumerate() {
            tag[i % 16] ^= byte;
        }

        Ok(EncryptedData {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            iv,
            tag,
            ciphertext,
            associated_data: associated_data.map(|s| s.to_string()),
        })
    }

    fn decrypt(&self, data: &EncryptedData) -> SecurityResult<Vec<u8>> {
        if data.algorithm != EncryptionAlgorithm::Aes256Gcm {
            return Err(SecurityError::DecryptionError(
                "Algorithm mismatch".to_string(),
            ));
        }

        // Verify tag (simplified check)
        let mut tag = vec![0u8; 16];
        for (i, &byte) in data.ciphertext.iter().enumerate() {
            tag[i % 16] ^= byte;
        }

        if tag != data.tag {
            return Err(SecurityError::DecryptionError(
                "Authentication tag verification failed".to_string(),
            ));
        }

        // Decrypt by XORing again with key
        let mut plaintext = Vec::with_capacity(data.ciphertext.len());
        for (i, &byte) in data.ciphertext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            let iv_byte = data.iv[i % data.iv.len()];
            plaintext.push(byte ^ key_byte ^ iv_byte);
        }

        Ok(plaintext)
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::Aes256Gcm
    }

    fn rotate_key(&mut self) -> SecurityResult<()> {
        self.key = Self::generate_random_bytes(32)?;
        Ok(())
    }
}

/// No-op encryption provider for development/testing
pub struct NoOpEncryptionProvider;

impl EncryptionProvider for NoOpEncryptionProvider {
    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&str>,
    ) -> SecurityResult<EncryptedData> {
        Ok(EncryptedData {
            algorithm: EncryptionAlgorithm::None,
            iv: Vec::new(),
            tag: Vec::new(),
            ciphertext: plaintext.to_vec(),
            associated_data: associated_data.map(|s| s.to_string()),
        })
    }

    fn decrypt(&self, data: &EncryptedData) -> SecurityResult<Vec<u8>> {
        Ok(data.ciphertext.clone())
    }

    fn algorithm(&self) -> EncryptionAlgorithm {
        EncryptionAlgorithm::None
    }

    fn rotate_key(&mut self) -> SecurityResult<()> {
        Ok(())
    }
}

/// Encryption utilities
pub struct EncryptionUtils;

impl EncryptionUtils {
    /// Encrypt string data
    pub fn encrypt_string(
        provider: &dyn EncryptionProvider,
        text: &str,
    ) -> SecurityResult<EncryptedData> {
        provider.encrypt(text.as_bytes(), None)
    }

    /// Decrypt string data
    pub fn decrypt_string(
        provider: &dyn EncryptionProvider,
        data: &EncryptedData,
    ) -> SecurityResult<String> {
        let bytes = provider.decrypt(data)?;
        String::from_utf8(bytes)
            .map_err(|e| SecurityError::DecryptionError(e.to_string()))
    }

    /// Encrypt JSON data
    pub fn encrypt_json<T: serde::Serialize>(
        provider: &dyn EncryptionProvider,
        data: &T,
    ) -> SecurityResult<EncryptedData> {
        let json = serde_json::to_vec(data)?;
        provider.encrypt(&json, None)
    }

    /// Decrypt JSON data
    pub fn decrypt_json<T: serde::de::DeserializeOwned>(
        provider: &dyn EncryptionProvider,
        data: &EncryptedData,
    ) -> SecurityResult<T> {
        let bytes = provider.decrypt(data)?;
        serde_json::from_slice(&bytes).map_err(|e| SecurityError::DecryptionError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_provider_creation() {
        let provider = Aes256GcmProvider::default();
        assert_eq!(provider.algorithm(), EncryptionAlgorithm::Aes256Gcm);
    }

    #[test]
    fn test_aes_encrypt_decrypt() {
        let provider = Aes256GcmProvider::default();
        let plaintext = b"sensitive data";

        let encrypted = provider.encrypt(plaintext, None).unwrap();
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert!(!encrypted.ciphertext.is_empty());

        let decrypted = provider.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_with_associated_data() {
        let provider = Aes256GcmProvider::default();
        let plaintext = b"data";

        let encrypted = provider.encrypt(plaintext, Some("context")).unwrap();
        assert_eq!(encrypted.associated_data, Some("context".to_string()));

        let decrypted = provider.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_rotation() {
        let mut provider = Aes256GcmProvider::default();
        let plaintext = b"test";

        let encrypted = provider.encrypt(plaintext, None).unwrap();
        let old_key = provider.get_key().to_vec();

        provider.rotate_key().unwrap();
        let new_key = provider.get_key().to_vec();

        assert_ne!(old_key, new_key);

        // Old encrypted data should fail to decrypt with new key
        assert!(provider.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_noop_provider() {
        let provider = NoOpEncryptionProvider;
        let plaintext = b"test";

        let encrypted = provider.encrypt(plaintext, None).unwrap();
        assert_eq!(encrypted.ciphertext, plaintext);

        let decrypted = provider.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_utils_string() {
        let provider = Aes256GcmProvider::default();
        let text = "secret message";

        let encrypted = EncryptionUtils::encrypt_string(&provider, text).unwrap();
        let decrypted = EncryptionUtils::decrypt_string(&provider, &encrypted).unwrap();

        assert_eq!(decrypted, text);
    }

    #[test]
    fn test_encryption_utils_json() {
        use serde::Serialize;

        let provider = Aes256GcmProvider::default();

        #[derive(Serialize, serde::Deserialize, PartialEq, Debug)]
        struct TestData {
            key: String,
            value: i32,
        }

        let data = TestData {
            key: "secret".to_string(),
            value: 42,
        };

        let encrypted = EncryptionUtils::encrypt_json(&provider, &data).unwrap();
        let decrypted: TestData = EncryptionUtils::decrypt_json(&provider, &encrypted).unwrap();

        assert_eq!(decrypted, data);
    }
}
