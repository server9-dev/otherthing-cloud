//! Example demonstrating the integration connector framework
//!
//! This example shows how to:
//! - Create and configure connectors
//! - Register connectors with the registry
//! - Execute connector requests
//! - Handle errors and retries
//! - Monitor connector health

use abcdodaf::integration::connector::{
    ApiKeyConfig, AuthConfig, Connector, ConnectorConfig, ConnectorRegistry, ConnectorRequest,
};
use abcdodaf::integration::connectors::{
    FileSystemConnector, OutgoingWebhookConnector, PostgresConnector, RestApiConnector,
};
use serde_json::json;
use std::sync::Arc;

/// Example connector factory for REST API
struct RestApiFactory;

#[async_trait::async_trait]
impl abcdodaf::integration::connector::registry::ConnectorFactory for RestApiFactory {
    async fn create(
        &self,
        config: ConnectorConfig,
    ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
        Ok(Box::new(RestApiConnector::new(config)))
    }
}

/// Example connector factory for PostgreSQL
struct PostgresFactory;

#[async_trait::async_trait]
impl abcdodaf::integration::connector::registry::ConnectorFactory for PostgresFactory {
    async fn create(
        &self,
        config: ConnectorConfig,
    ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
        Ok(Box::new(PostgresConnector::new(config)))
    }
}

/// Example connector factory for FileSystem
struct FileSystemFactory;

#[async_trait::async_trait]
impl abcdodaf::integration::connector::registry::ConnectorFactory for FileSystemFactory {
    async fn create(
        &self,
        config: ConnectorConfig,
    ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
        Ok(Box::new(FileSystemConnector::new(config)))
    }
}

/// Example connector factory for Webhooks
struct WebhookFactory;

#[async_trait::async_trait]
impl abcdodaf::integration::connector::registry::ConnectorFactory for WebhookFactory {
    async fn create(
        &self,
        config: ConnectorConfig,
    ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
        Ok(Box::new(OutgoingWebhookConnector::new(config)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ABCDODAF Connector Framework Example");
    println!("====================================\n");

    // Create connector registry
    let registry = ConnectorRegistry::new();

    // Register factories for different connector types
    registry.register_factory("rest_api", Arc::new(RestApiFactory)).await;
    registry.register_factory("postgresql", Arc::new(PostgresFactory)).await;
    registry.register_factory("filesystem", Arc::new(FileSystemFactory)).await;
    registry.register_factory("outgoing_webhook", Arc::new(WebhookFactory)).await;

    println!("Registered {} connector factories\n", registry.factory_count().await);

    // Example 1: Create a REST API connector with API key authentication
    println!("Example 1: REST API Connector");
    println!("----------------------------");
    let api_config = ConnectorConfig::new("github_api", "rest_api")
        .with_param("url", json!("https://api.github.com"))
        .with_auth(AuthConfig::ApiKey(
            ApiKeyConfig::new("ghp_xxxx", "Authorization").with_prefix("Bearer "),
        ))
        .with_timeout(30)
        .with_retries(true, 3);

    let api_connector = registry.create_connector(api_config).await?;
    println!("Created REST API connector: github_api");

    let connector = api_connector.read().await;
    println!("Status: {}", connector.status());
    println!("Type: {}\n", connector.connector_type());

    // Example 2: Create a PostgreSQL connector
    println!("Example 2: PostgreSQL Connector");
    println!("-------------------------------");
    let db_config = ConnectorConfig::new("main_db", "postgresql")
        .with_param("host", json!("localhost"))
        .with_param("port", json!(5432))
        .with_param("database", json!("myapp"))
        .with_param("user", json!("postgres"))
        .with_param("password", json!("secret"))
        .with_pooling(true, 20)
        .with_circuit_breaker(true, 5, 60);

    let db_connector = registry.create_connector(db_config).await?;
    println!("Created PostgreSQL connector: main_db");

    let connector = db_connector.read().await;
    println!("Status: {}", connector.status());
    println!("Type: {}\n", connector.connector_type());

    // Example 3: Create a FileSystem connector
    println!("Example 3: FileSystem Connector");
    println!("-------------------------------");
    let fs_config = ConnectorConfig::new("data_storage", "filesystem")
        .with_param("base_path", json!("/tmp/abcdodaf"))
        .with_config("allow_read", json!(true))
        .with_config("allow_write", json!(true))
        .with_config("allow_delete", json!(false));

    let fs_connector = registry.create_connector(fs_config).await?;
    println!("Created FileSystem connector: data_storage");

    let connector = fs_connector.read().await;
    println!("Status: {}", connector.status());
    println!("Type: {}\n", connector.connector_type());

    // Example 4: Create a Webhook connector
    println!("Example 4: Webhook Connector");
    println!("----------------------------");
    let webhook_config = ConnectorConfig::new("slack_webhook", "outgoing_webhook")
        .with_param("url", json!("https://hooks.slack.com/services/xxx"))
        .with_config("retry_on_failure", json!(true))
        .with_timeout(15);

    let webhook_connector = registry.create_connector(webhook_config).await?;
    println!("Created Webhook connector: slack_webhook");

    let connector = webhook_connector.read().await;
    println!("Status: {}", connector.status());
    println!("Type: {}\n", connector.connector_type());

    // Example 5: List all registered connectors
    println!("Example 5: Registered Connectors");
    println!("--------------------------------");
    let connectors = registry.list_connectors().await;
    for name in &connectors {
        println!("  - {}", name);
    }
    println!("Total: {}\n", connectors.len());

    // Example 6: Health check all connectors
    println!("Example 6: Connector Health Status");
    println!("----------------------------------");
    let health_results = registry.health_check_all().await;
    for (name, status, _result) in health_results {
        let status_str = match status {
            abcdodaf::integration::connector::HealthStatus::Healthy => "Healthy",
            abcdodaf::integration::connector::HealthStatus::Degraded(msg) => {
                println!("  {} - Degraded: {}", name, msg);
                continue;
            },
            abcdodaf::integration::connector::HealthStatus::Unhealthy(msg) => {
                println!("  {} - Unhealthy: {}", name, msg);
                continue;
            },
        };
        println!("  {} - {}", name, status_str);
    }
    println!();

    // Example 7: Execute a request through a connector
    println!("Example 7: Execute Connector Request");
    println!("------------------------------------");
    let request = ConnectorRequest::new("POST")
        .with_param("id", json!("123"))
        .with_body(json!({
            "title": "New Task",
            "description": "Task created via connector"
        }))
        .with_timeout(10);

    println!("Request ID: {}", request.id);
    println!("Operation: {}", request.operation);
    println!("Timeout: {}s\n", request.timeout_secs.unwrap_or(0));

    // Example 8: Demonstrate retry policy
    println!("Example 8: Retry Policy");
    println!("-----------------------");
    use abcdodaf::integration::connector::RetryPolicy;
    use std::time::Duration;

    let retry_policy =
        RetryPolicy::exponential_backoff(4, Duration::from_millis(100), Duration::from_secs(30));

    println!("Exponential Backoff Retry Policy:");
    for attempt in 0..4 {
        if let Some(delay) = retry_policy.next_retry_delay(attempt) {
            println!("  Attempt {}: Wait {}ms before retry", attempt + 1, delay.as_millis());
        }
    }
    println!();

    // Example 9: Demonstrate circuit breaker
    println!("Example 9: Circuit Breaker");
    println!("--------------------------");
    use abcdodaf::integration::connector::CircuitBreaker;

    let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(60));
    println!("Initial state: {}", circuit_breaker.state());

    circuit_breaker.record_failure();
    circuit_breaker.record_failure();
    println!("After 2 failures: {}", circuit_breaker.state());

    circuit_breaker.record_failure();
    println!("After 3 failures (threshold): {}", circuit_breaker.state());
    println!();

    // Example 10: Clean up
    println!("Example 10: Cleanup");
    println!("-------------------");
    println!("Closing all connectors...");
    registry.close_all().await?;
    println!("All connectors closed successfully");

    Ok(())
}
