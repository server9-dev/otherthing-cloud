//! Authentication strategies for connectors

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication strategy trait
pub trait AuthStrategy: Send + Sync {
    /// Apply authentication to headers
    fn apply(&self, headers: &mut HashMap<String, String>);

    /// Check if credentials are valid
    fn is_valid(&self) -> bool;

    /// Refresh credentials if needed
    fn refresh(&mut self) -> Result<(), String>;
}

/// API Key authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key value
    pub key: String,
    /// Header name (e.g., "X-API-Key", "Authorization")
    pub header_name: String,
    /// Optional prefix (e.g., "Bearer ", "ApiKey ")
    pub prefix: Option<String>,
}

impl ApiKeyConfig {
    /// Create a new API key configuration
    pub fn new(key: impl Into<String>, header_name: impl Into<String>) -> Self {
        Self { key: key.into(), header_name: header_name.into(), prefix: None }
    }

    /// Set a prefix for the key
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

impl AuthStrategy for ApiKeyConfig {
    fn apply(&self, headers: &mut HashMap<String, String>) {
        let value = if let Some(ref prefix) = self.prefix {
            format!("{}{}", prefix, self.key)
        } else {
            self.key.clone()
        };
        headers.insert(self.header_name.clone(), value);
    }

    fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    fn refresh(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// OAuth 2.0 authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth client ID
    pub client_id: String,
    /// OAuth client secret
    pub client_secret: String,
    /// OAuth token endpoint URL
    pub token_endpoint: String,
    /// Scopes requested
    pub scopes: Vec<String>,
    /// Current access token
    pub access_token: Option<String>,
    /// Token expiration Unix timestamp
    pub token_expires_at: Option<i64>,
}

impl OAuthConfig {
    /// Create a new OAuth configuration
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            token_endpoint: token_endpoint.into(),
            scopes: Vec::new(),
            access_token: None,
            token_expires_at: None,
        }
    }

    /// Add a scope
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Check if token is expired
    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                now >= expires_at
            },
            None => true,
        }
    }

    /// Set the access token
    pub fn set_token(mut self, token: impl Into<String>, expires_in_secs: u64) -> Self {
        self.access_token = Some(token.into());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.token_expires_at = Some(now + expires_in_secs as i64);
        self
    }
}

impl AuthStrategy for OAuthConfig {
    fn apply(&self, headers: &mut HashMap<String, String>) {
        if let Some(ref token) = self.access_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
    }

    fn is_valid(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty() && self.access_token.is_some()
    }

    fn refresh(&mut self) -> Result<(), String> {
        if self.is_token_expired() {
            return Err("Token expired and refresh not implemented".to_string());
        }
        Ok(())
    }
}

/// JWT (JSON Web Token) authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT token
    pub token: String,
    /// Token expiration Unix timestamp
    pub token_expires_at: Option<i64>,
    /// Header name (usually "Authorization")
    pub header_name: String,
}

impl JwtConfig {
    /// Create a new JWT configuration
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            token_expires_at: None,
            header_name: "Authorization".to_string(),
        }
    }

    /// Set token expiration
    pub fn with_expiration(mut self, expires_at: i64) -> Self {
        self.token_expires_at = Some(expires_at);
        self
    }

    /// Check if token is expired
    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                now >= expires_at
            },
            None => false,
        }
    }
}

impl AuthStrategy for JwtConfig {
    fn apply(&self, headers: &mut HashMap<String, String>) {
        headers.insert(self.header_name.clone(), format!("Bearer {}", self.token));
    }

    fn is_valid(&self) -> bool {
        !self.token.is_empty() && !self.is_token_expired()
    }

    fn refresh(&mut self) -> Result<(), String> {
        if self.is_token_expired() {
            return Err("JWT token expired and refresh not implemented".to_string());
        }
        Ok(())
    }
}

/// Basic authentication (username:password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

impl BasicAuthConfig {
    /// Create a new basic auth configuration
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self { username: username.into(), password: password.into() }
    }
}

impl AuthStrategy for BasicAuthConfig {
    fn apply(&self, headers: &mut HashMap<String, String>) {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = base64_encode(&credentials);
        headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
    }

    fn is_valid(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty()
    }

    fn refresh(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// Encode string to base64
fn base64_encode(s: &str) -> String {
    const BASE64_TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let bytes = s.as_bytes();

    for chunk in bytes.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }

        let b1 = buf[0] >> 2;
        let b2 = ((buf[0] & 0x03) << 4) | (buf[1] >> 4);
        let b3 = ((buf[1] & 0x0f) << 2) | (buf[2] >> 6);
        let b4 = buf[2] & 0x3f;

        result.push(BASE64_TABLE[b1 as usize] as char);
        result.push(BASE64_TABLE[b2 as usize] as char);

        if chunk.len() > 1 {
            result.push(BASE64_TABLE[b3 as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(BASE64_TABLE[b4 as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth() {
        let auth = ApiKeyConfig::new("secret-key", "X-API-Key");
        let mut headers = HashMap::new();
        auth.apply(&mut headers);

        assert_eq!(headers.get("X-API-Key"), Some(&"secret-key".to_string()));
    }

    #[test]
    fn test_api_key_with_prefix() {
        let auth = ApiKeyConfig::new("token123", "Authorization").with_prefix("Bearer ");
        let mut headers = HashMap::new();
        auth.apply(&mut headers);

        assert_eq!(headers.get("Authorization"), Some(&"Bearer token123".to_string()));
    }

    #[test]
    fn test_jwt_auth() {
        let auth = JwtConfig::new("eyJhbGc...");
        let mut headers = HashMap::new();
        auth.apply(&mut headers);

        assert_eq!(headers.get("Authorization"), Some(&"Bearer eyJhbGc...".to_string()));
    }

    #[test]
    fn test_basic_auth() {
        let auth = BasicAuthConfig::new("user", "pass");
        let mut headers = HashMap::new();
        auth.apply(&mut headers);

        let auth_header = headers.get("Authorization").unwrap();
        assert!(auth_header.starts_with("Basic "));
    }

    #[test]
    fn test_oauth_token_expiration() {
        let _now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut oauth = OAuthConfig::new("client", "secret", "https://example.com/token");
        assert!(oauth.is_token_expired());

        oauth = oauth.set_token("token", 3600);
        assert!(!oauth.is_token_expired());
    }
}
