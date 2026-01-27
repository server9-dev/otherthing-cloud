//! REST API connector for HTTP-based integrations

use crate::integration::connector::{
    Connector, ConnectionStatus, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus, AuthConfig,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

/// REST API connector for HTTP integrations
pub struct RestApiConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    client: Option<reqwest::Client>,
}

impl RestApiConnector {
    /// Create a new REST API connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            client: None,
        }
    }

    /// Get the base URL
    fn get_base_url(&self) -> ConnectorResult<String> {
        self.config
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| ConnectorError::config("Base URL not configured"))
    }

    /// Build the full URL for a request
    fn build_url(&self, operation: &str) -> ConnectorResult<String> {
        let base_url = self.get_base_url()?;
        Ok(format!("{}{}", base_url, operation))
    }

    /// Map HTTP method string to reqwest method
    fn get_http_method(&self, method: &str) -> ConnectorResult<reqwest::Method> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(reqwest::Method::GET),
            "POST" => Ok(reqwest::Method::POST),
            "PUT" => Ok(reqwest::Method::PUT),
            "DELETE" => Ok(reqwest::Method::DELETE),
            "PATCH" => Ok(reqwest::Method::PATCH),
            "HEAD" => Ok(reqwest::Method::HEAD),
            "OPTIONS" => Ok(reqwest::Method::OPTIONS),
            _ => Err(ConnectorError::validation(format!(
                "Unsupported HTTP method: {}",
                method
            ))),
        }
    }

    /// Apply authentication to request headers
    fn apply_auth(&self, headers: &mut reqwest::header::HeaderMap) -> ConnectorResult<()> {
        use crate::integration::connector::auth::AuthStrategy;

        match &self.config.auth {
            AuthConfig::None => Ok(()),
            AuthConfig::ApiKey(auth) => {
                let mut map = HashMap::new();
                auth.apply(&mut map);
                for (key, value) in map {
                    headers.insert(
                        key.parse::<reqwest::header::HeaderName>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                        value
                            .parse::<reqwest::header::HeaderValue>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                    );
                }
                Ok(())
            }
            AuthConfig::Jwt(auth) => {
                let mut map = HashMap::new();
                auth.apply(&mut map);
                for (key, value) in map {
                    headers.insert(
                        key.parse::<reqwest::header::HeaderName>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                        value
                            .parse::<reqwest::header::HeaderValue>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                    );
                }
                Ok(())
            }
            AuthConfig::Basic(auth) => {
                let mut map = HashMap::new();
                auth.apply(&mut map);
                for (key, value) in map {
                    headers.insert(
                        key.parse::<reqwest::header::HeaderName>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                        value
                            .parse::<reqwest::header::HeaderValue>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                    );
                }
                Ok(())
            }
            AuthConfig::OAuth2(auth) => {
                let mut map = HashMap::new();
                auth.apply(&mut map);
                for (key, value) in map {
                    headers.insert(
                        key.parse::<reqwest::header::HeaderName>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                        value
                            .parse::<reqwest::header::HeaderValue>()
                            .map_err(|e| ConnectorError::config(e.to_string()))?,
                    );
                }
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Connector for RestApiConnector {
    fn connector_type(&self) -> &str {
        "rest_api"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify URL is configured
        self.get_base_url()?;

        // Create HTTP client
        self.client = Some(
            reqwest::Client::builder()
                .timeout(self.config.get_timeout())
                .build()
                .map_err(|e| ConnectorError::connection(e.to_string()))?,
        );

        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.client = None;
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection(format!(
                "Connector not connected: {}",
                self.status
            )));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ConnectorError::connection("HTTP client not initialized"))?;

        let url = self.build_url(&request.operation)?;
        let method = self.get_http_method(&request.operation)?;

        let start = Instant::now();

        // Build request
        let mut req = client.request(method, &url);

        // Add query parameters
        for (key, value) in &request.parameters {
            req = req.query(&[(key, value.to_string())]);
        }

        // Add body
        if let Some(body) = &request.body {
            req = req.json(body);
        }

        // Add headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &request.headers {
            headers.insert(
                key.parse::<reqwest::header::HeaderName>()
                    .map_err(|e| ConnectorError::config(e.to_string()))?,
                value
                    .parse::<reqwest::header::HeaderValue>()
                    .map_err(|e| ConnectorError::config(e.to_string()))?,
            );
        }

        // Apply authentication
        self.apply_auth(&mut headers)?;

        req = req.headers(headers);

        // Execute request
        let response = req
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ConnectorError::timeout(self.config.timeout_secs.unwrap_or(30))
                } else {
                    ConnectorError::request(e.to_string())
                }
            })?;

        let status_code = response.status().as_u16();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();

        let body = response
            .json::<Value>()
            .await
            .unwrap_or(serde_json::json!({}));

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectorResponse::new(&request.id, status_code)
            .with_body(body)
            .with_execution_time(execution_time_ms)
            .with_header("content_type", response_headers.get("content-type").cloned().unwrap_or_default()))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status != ConnectionStatus::Connected {
            return Ok(HealthStatus::Unhealthy(format!(
                "Connector status: {}",
                self.status
            )));
        }

        // Try to make a HEAD request to the base URL
        if let Ok(url) = self.get_base_url() {
            let client = match &self.client {
                Some(c) => c,
                None => {
                    return Ok(HealthStatus::Unhealthy(
                        "HTTP client not initialized".to_string(),
                    ))
                }
            };

            match client.head(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(HealthStatus::Healthy)
                    } else {
                        Ok(HealthStatus::Degraded(format!(
                            "Server returned status {}",
                            response.status()
                        )))
                    }
                }
                Err(e) => Ok(HealthStatus::Unhealthy(e.to_string())),
            }
        } else {
            Ok(HealthStatus::Unhealthy("Base URL not configured".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_connector_creation() {
        let config = ConnectorConfig::new("test_api", "rest_api")
            .with_param("url", serde_json::json!("https://api.example.com"));
        let connector = RestApiConnector::new(config);

        assert_eq!(connector.connector_type(), "rest_api");
        assert_eq!(connector.name(), "test_api");
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_http_method_parsing() {
        let config = ConnectorConfig::new("test", "rest_api")
            .with_param("url", serde_json::json!("https://example.com"));
        let connector = RestApiConnector::new(config);

        assert_eq!(connector.get_http_method("GET").unwrap(), reqwest::Method::GET);
        assert_eq!(connector.get_http_method("POST").unwrap(), reqwest::Method::POST);
        assert_eq!(connector.get_http_method("DELETE").unwrap(), reqwest::Method::DELETE);
        assert!(connector.get_http_method("INVALID").is_err());
    }

    #[test]
    fn test_url_building() {
        let config = ConnectorConfig::new("test", "rest_api")
            .with_param("url", serde_json::json!("https://api.example.com/v1"));
        let connector = RestApiConnector::new(config);

        let url = connector.build_url("/users").unwrap();
        assert_eq!(url, "https://api.example.com/v1/users");
    }
}
