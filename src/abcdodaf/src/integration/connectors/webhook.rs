//! Webhook connectors for incoming and outgoing webhooks

use crate::integration::connector::{
    Connector, ConnectionStatus, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

/// Webhook event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Event timestamp
    pub timestamp: i64,
    /// Event payload
    pub payload: Value,
    /// Event headers
    pub headers: HashMap<String, String>,
}

impl WebhookEvent {
    /// Create a new webhook event
    pub fn new(event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            payload,
            headers: HashMap::new(),
        }
    }

    /// Add a header to the event
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Outgoing webhook connector for sending webhooks
pub struct OutgoingWebhookConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    client: Option<reqwest::Client>,
}

impl OutgoingWebhookConnector {
    /// Create a new outgoing webhook connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            client: None,
        }
    }

    /// Get the webhook URL
    fn get_webhook_url(&self) -> ConnectorResult<String> {
        self.config
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| ConnectorError::config("Webhook URL not configured"))
    }

    /// Get retry on failure setting
    fn should_retry_on_failure(&self) -> bool {
        self.config
            .config
            .get("retry_on_failure")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// Get timeout in seconds
    fn get_timeout_secs(&self) -> u64 {
        self.config.timeout_secs.unwrap_or(30)
    }
}

#[async_trait]
impl Connector for OutgoingWebhookConnector {
    fn connector_type(&self) -> &str {
        "outgoing_webhook"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify webhook URL is configured
        self.get_webhook_url()?;

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
            return Err(ConnectorError::connection("Webhook connector not connected"));
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| ConnectorError::connection("HTTP client not initialized"))?;

        let url = self.get_webhook_url()?;
        let start = Instant::now();

        // Build webhook event
        let event = match request.body {
            Some(payload) => WebhookEvent::new(&request.operation, payload),
            None => WebhookEvent::new(&request.operation, json!({})),
        };

        // Send webhook
        let response = client
            .post(&url)
            .json(&event)
            .timeout(self.config.get_timeout())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ConnectorError::timeout(self.get_timeout_secs())
                } else {
                    ConnectorError::request(e.to_string())
                }
            })?;

        let status_code = response.status().as_u16();
        let body = response
            .json::<Value>()
            .await
            .unwrap_or(json!({}));

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectorResponse::new(&request.id, status_code)
            .with_body(json!({
                "event_id": event.id,
                "event_type": event.event_type,
                "response": body
            }))
            .with_execution_time(execution_time_ms))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status != ConnectionStatus::Connected {
            return Ok(HealthStatus::Unhealthy(format!(
                "Webhook connector status: {}",
                self.status
            )));
        }

        // Verify webhook URL is accessible
        if let Ok(url) = self.get_webhook_url() {
            if let Some(client) = &self.client {
                match client.head(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            Ok(HealthStatus::Healthy)
                        } else {
                            Ok(HealthStatus::Degraded(format!(
                                "Webhook returned status {}",
                                response.status()
                            )))
                        }
                    }
                    Err(e) => Ok(HealthStatus::Unhealthy(e.to_string())),
                }
            } else {
                Ok(HealthStatus::Unhealthy("HTTP client not initialized".to_string()))
            }
        } else {
            Ok(HealthStatus::Unhealthy("Webhook URL not configured".to_string()))
        }
    }
}

/// Incoming webhook listener (minimal implementation)
pub struct IncomingWebhookConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    events: Vec<WebhookEvent>,
}

impl IncomingWebhookConnector {
    /// Create a new incoming webhook connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            events: Vec::new(),
        }
    }

    /// Get the listening port
    fn get_port(&self) -> ConnectorResult<u16> {
        self.config
            .params
            .get("port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16)
            .ok_or_else(|| ConnectorError::config("Port not configured"))
    }

    /// Get the webhook path
    fn get_path(&self) -> String {
        self.config
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/webhook")
            .to_string()
    }
}

#[async_trait]
impl Connector for IncomingWebhookConnector {
    fn connector_type(&self) -> &str {
        "incoming_webhook"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify port is configured
        self.get_port()?;

        // In a real implementation, this would start an HTTP server
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.status = ConnectionStatus::Closed;
        self.events.clear();
        Ok(())
    }

    async fn execute(&self, _request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection("Webhook listener not listening"));
        }

        // Return received events
        let body = json!({
            "event_count": self.events.len(),
            "events": self.events.iter().map(|e| json!({
                "id": e.id,
                "type": e.event_type,
                "timestamp": e.timestamp
            })).collect::<Vec<_>>()
        });

        Ok(ConnectorResponse::new("webhook_query", 200).with_body(body))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status == ConnectionStatus::Connected {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy(format!(
                "Webhook listener status: {}",
                self.status
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEvent::new("user.created", json!({"user_id": "123"}))
            .with_header("X-Custom", "value");

        assert_eq!(event.event_type, "user.created");
        assert_eq!(
            event.payload.get("user_id"),
            Some(&json!("123"))
        );
        assert_eq!(event.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_outgoing_webhook_connector_creation() {
        let config = ConnectorConfig::new("webhook_out", "outgoing_webhook")
            .with_param("url", json!("https://example.com/webhook"));

        let connector = OutgoingWebhookConnector::new(config);
        assert_eq!(connector.connector_type(), "outgoing_webhook");
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_incoming_webhook_connector_creation() {
        let config = ConnectorConfig::new("webhook_in", "incoming_webhook")
            .with_param("port", json!(8080))
            .with_param("path", json!("/webhook"));

        let connector = IncomingWebhookConnector::new(config);
        assert_eq!(connector.connector_type(), "incoming_webhook");
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);
        assert_eq!(connector.get_port().unwrap(), 8080);
        assert_eq!(connector.get_path(), "/webhook");
    }
}
