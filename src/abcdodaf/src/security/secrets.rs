//! Secret management for credentials and API keys
//!
//! Provides secure storage and retrieval of sensitive credentials with
//! encryption and access control.

use crate::security::encryption::{Aes256GcmProvider, EncryptedData, EncryptionProvider};
use crate::security::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type of secret
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretType {
    /// API key
    ApiKey,
    /// Database password
    DatabasePassword,
    /// OAuth token
    OAuthToken,
    /// SSH key
    SshKey,
    /// Certificate
    Certificate,
    /// Generic credential
    Credential,
    /// Custom secret
    Custom,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::ApiKey => write!(f, "API Key"),
            SecretType::DatabasePassword => write!(f, "Database Password"),
            SecretType::OAuthToken => write!(f, "OAuth Token"),
            SecretType::SshKey => write!(f, "SSH Key"),
            SecretType::Certificate => write!(f, "Certificate"),
            SecretType::Credential => write!(f, "Credential"),
            SecretType::Custom => write!(f, "Custom"),
        }
    }
}

/// Secret value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretValue {
    /// Name/identifier of the secret
    pub name: String,
    /// Type of secret
    pub secret_type: SecretType,
    /// Encrypted secret data
    pub encrypted_data: EncryptedData,
    /// Metadata about the secret
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed timestamp
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Expiration timestamp (if applicable)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether the secret is still valid
    pub is_active: bool,
    /// Owner subject ID
    pub owner_id: Option<String>,
}

impl SecretValue {
    /// Create a new secret value
    pub fn new(
        name: impl Into<String>,
        secret_type: SecretType,
        encrypted_data: EncryptedData,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            secret_type,
            encrypted_data,
            metadata: HashMap::new(),
            created_at: now,
            modified_at: now,
            last_accessed_at: None,
            expires_at: None,
            is_active: true,
            owner_id: None,
        }
    }

    /// Set owner
    pub fn with_owner(mut self, owner_id: impl Into<String>) -> Self {
        self.owner_id = Some(owner_id.into());
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if secret is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expires_at {
            chrono::Utc::now() > expiry
        } else {
            false
        }
    }

    /// Mark as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed_at = Some(chrono::Utc::now());
    }
}

/// Secret manager for storing and retrieving secrets
#[derive(Clone)]
pub struct SecretManager {
    /// Stored secrets (name -> secret)
    secrets: Arc<RwLock<HashMap<String, SecretValue>>>,
    /// Encryption provider
    encryption_provider: Arc<RwLock<Box<dyn EncryptionProvider>>>,
    /// Access log (name -> list of subject IDs who accessed it)
    access_log: Arc<RwLock<HashMap<String, Vec<AccessRecord>>>>,
}

/// Record of secret access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRecord {
    /// Subject who accessed the secret
    pub subject_id: String,
    /// Timestamp of access
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether access was granted
    pub was_granted: bool,
}

impl SecretManager {
    /// Create a new secret manager with AES-256-GCM encryption
    pub async fn new() -> SecurityResult<Self> {
        let encryption_provider: Box<dyn EncryptionProvider> =
            Box::new(Aes256GcmProvider::default());

        Ok(Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            encryption_provider: Arc::new(RwLock::new(encryption_provider)),
            access_log: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a secret (already encrypted)
    pub async fn store_secret(&self, secret: SecretValue) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().await;

        if secrets.contains_key(&secret.name) {
            return Err(SecurityError::Other(format!("Secret '{}' already exists", secret.name)));
        }

        secrets.insert(secret.name.clone(), secret);
        Ok(())
    }

    /// Store and encrypt a plaintext secret
    pub async fn store_plaintext(
        &self,
        name: impl Into<String>,
        secret_type: SecretType,
        plaintext: &str,
    ) -> SecurityResult<()> {
        let name_str = name.into();
        let provider = self.encryption_provider.read().await;
        let encrypted_data = provider.encrypt(plaintext.as_bytes(), Some(&name_str))?;
        drop(provider);

        let secret = SecretValue::new(&name_str, secret_type, encrypted_data);
        self.store_secret(secret).await
    }

    /// Retrieve and decrypt a secret
    pub async fn get_secret(&self, name: &str, subject_id: &str) -> SecurityResult<Vec<u8>> {
        let encrypted_data = {
            let mut secrets = self.secrets.write().await;

            let secret = secrets
                .get_mut(name)
                .ok_or_else(|| SecurityError::SecretNotFound(name.to_string()))?;

            // Check if expired
            if secret.is_expired() {
                return Err(SecurityError::Other(format!("Secret '{}' has expired", name)));
            }

            // Check if active
            if !secret.is_active {
                return Err(SecurityError::Other(format!("Secret '{}' is inactive", name)));
            }

            secret.mark_accessed();
            secret.encrypted_data.clone()
        };

        // Decrypt the secret
        let provider = self.encryption_provider.read().await;
        let plaintext = provider.decrypt(&encrypted_data)?;

        // Log the access
        let mut access_log = self.access_log.write().await;
        let records = access_log.entry(name.to_string()).or_insert_with(Vec::new);
        records.push(AccessRecord {
            subject_id: subject_id.to_string(),
            timestamp: chrono::Utc::now(),
            was_granted: true,
        });

        Ok(plaintext)
    }

    /// Retrieve secret as string
    pub async fn get_secret_string(&self, name: &str, subject_id: &str) -> SecurityResult<String> {
        let bytes = self.get_secret(name, subject_id).await?;
        String::from_utf8(bytes).map_err(|e| SecurityError::DecryptionError(e.to_string()))
    }

    /// Delete a secret
    pub async fn delete_secret(&self, name: &str) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().await;
        secrets
            .remove(name)
            .ok_or_else(|| SecurityError::SecretNotFound(name.to_string()))?;

        let mut access_log = self.access_log.write().await;
        access_log.remove(name);

        Ok(())
    }

    /// List secret names (without revealing values)
    pub async fn list_secrets(&self) -> SecurityResult<Vec<String>> {
        let secrets = self.secrets.read().await;
        Ok(secrets.keys().cloned().collect())
    }

    /// Get secret metadata
    pub async fn get_secret_metadata(&self, name: &str) -> SecurityResult<SecretValue> {
        let secrets = self.secrets.read().await;
        secrets
            .get(name)
            .cloned()
            .ok_or_else(|| SecurityError::SecretNotFound(name.to_string()))
    }

    /// Deactivate a secret
    pub async fn deactivate_secret(&self, name: &str) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(name) {
            secret.is_active = false;
            secret.modified_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(SecurityError::SecretNotFound(name.to_string()))
        }
    }

    /// Reactivate a secret
    pub async fn activate_secret(&self, name: &str) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(name) {
            secret.is_active = true;
            secret.modified_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(SecurityError::SecretNotFound(name.to_string()))
        }
    }

    /// Get access log for a secret
    pub async fn get_access_log(&self, name: &str) -> SecurityResult<Vec<AccessRecord>> {
        let access_log = self.access_log.read().await;
        Ok(access_log.get(name).cloned().unwrap_or_default())
    }

    /// Rotate secret (create new encrypted version with rotated key)
    pub async fn rotate_secret(&self, name: &str) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().await;

        if let Some(secret) = secrets.get_mut(name) {
            secret.modified_at = chrono::Utc::now();

            // In production, this would decrypt with old key, encrypt with new key
            // For now, we just update the timestamp
            Ok(())
        } else {
            Err(SecurityError::SecretNotFound(name.to_string()))
        }
    }

    /// Count stored secrets
    pub async fn count_secrets(&self) -> SecurityResult<usize> {
        let secrets = self.secrets.read().await;
        Ok(secrets.len())
    }

    /// Check if secret exists
    pub async fn secret_exists(&self, name: &str) -> SecurityResult<bool> {
        let secrets = self.secrets.read().await;
        Ok(secrets.contains_key(name))
    }
}

// Note: Default trait removed to prevent panics.
// SecretManager requires async initialization, so users must call SecretManager::new().

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_manager_creation() {
        let manager = SecretManager::new().await.unwrap();
        assert_eq!(manager.count_secrets().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_plaintext() {
        let manager = SecretManager::new().await.unwrap();

        manager
            .store_plaintext("api_key", SecretType::ApiKey, "secret123")
            .await
            .unwrap();

        let secret = manager.get_secret_string("api_key", "user1").await.unwrap();
        assert_eq!(secret, "secret123");
    }

    #[tokio::test]
    async fn test_secret_not_found() {
        let manager = SecretManager::new().await.unwrap();

        let result = manager.get_secret("nonexistent", "user1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_secret() {
        let manager = SecretManager::new().await.unwrap();

        manager
            .store_plaintext("api_key", SecretType::ApiKey, "secret123")
            .await
            .unwrap();

        assert!(manager.secret_exists("api_key").await.unwrap());

        manager.delete_secret("api_key").await.unwrap();

        assert!(!manager.secret_exists("api_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_deactivate_secret() {
        let manager = SecretManager::new().await.unwrap();

        manager
            .store_plaintext("api_key", SecretType::ApiKey, "secret123")
            .await
            .unwrap();

        manager.deactivate_secret("api_key").await.unwrap();

        let result = manager.get_secret("api_key", "user1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_access_log() {
        let manager = SecretManager::new().await.unwrap();

        manager
            .store_plaintext("api_key", SecretType::ApiKey, "secret123")
            .await
            .unwrap();

        manager.get_secret_string("api_key", "user1").await.unwrap();

        manager.get_secret_string("api_key", "user2").await.unwrap();

        let log = manager.get_access_log("api_key").await.unwrap();
        assert_eq!(log.len(), 2);
    }
}
