# ABCDODAF Integration Layer Guide

## Overview

The ABCDODAF integration layer provides an extensible framework for connecting workflows to external systems. It supports REST APIs, databases, webhooks, file system operations, and custom connectors.

## Architecture

### Core Components

1. **Connector Framework** - Base traits and types for building connectors
2. **Connector Implementations** - Pre-built connectors for common use cases
3. **Connector Registry** - Centralized management of connector instances
4. **Authentication** - Pluggable authentication strategies (API Keys, OAuth2, JWT, Basic Auth)
5. **Resilience** - Retry policies and circuit breakers for fault tolerance
6. **Configuration** - Flexible connector configuration system

### Component Diagram

```
┌─────────────────────────────────────────────┐
│         Connector Registry                  │
│  (Manages all connector instances)          │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────┼──────────┬──────────────┐
        │          │          │              │
   ┌────▼──┐  ┌───▼───┐  ┌──▼──┐      ┌───▼────┐
   │ REST  │  │  DB   │  │ FSM │      │Webhooks│
   │ API   │  │(PG,MY)│  │ (*)│      │  (*) │
   └───────┘  └───────┘  └─────┘      └────────┘

(*) = Extensible via trait implementation
```

## Quick Start

### 1. Create and Configure a Connector

```rust
use abcdodaf::integration::{ConnectorConfig, AuthConfig};
use serde_json::json;

// Configure a REST API connector
let config = ConnectorConfig::new("github_api", "rest_api")
    .with_param("url", json!("https://api.github.com"))
    .with_auth(AuthConfig::api_key("your-token", "Authorization")
        .with_prefix("Bearer "))
    .with_timeout(30)
    .with_retries(true, 3)
    .with_circuit_breaker(true, 5, 60);
```

### 2. Register Connector Factory

```rust
use abcdodaf::integration::ConnectorRegistry;
use std::sync::Arc;

let registry = ConnectorRegistry::new();

// Register factory for REST API connectors
registry.register_factory("rest_api", Arc::new(RestApiFactory)).await;

// Create connector from config
let connector = registry.create_connector(config).await?;
```

### 3. Execute Requests

```rust
use abcdodaf::integration::ConnectorRequest;
use serde_json::json;

let request = ConnectorRequest::new("GET")
    .with_param("page", json!(1))
    .with_param("per_page", json!(10))
    .with_timeout(30);

let response = connector.execute(request).await?;
println!("Status: {}", response.status_code);
println!("Body: {}", response.body);
```

## Connector Types

### REST API Connector

For HTTP-based integrations (REST APIs, webhooks, etc.)

```rust
let config = ConnectorConfig::new("api", "rest_api")
    .with_param("url", json!("https://api.example.com/v1"))
    .with_auth(AuthConfig::api_key("key", "X-API-Key"));

let connector = RestApiConnector::new(config);
```

**Supported Operations:**
- GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS

### PostgreSQL Connector

For PostgreSQL database operations

```rust
let config = ConnectorConfig::new("db", "postgresql")
    .with_param("host", json!("localhost"))
    .with_param("port", json!(5432))
    .with_param("database", json!("mydb"))
    .with_param("user", json!("user"))
    .with_param("password", json!("pass"));

let connector = PostgresConnector::new(config);
```

**Supported Operations:**
- QUERY - Execute SELECT statements
- INSERT - Insert data
- UPDATE - Update data
- DELETE - Delete data

### MySQL Connector

Similar to PostgreSQL, for MySQL/MariaDB databases

```rust
let config = ConnectorConfig::new("db", "mysql")
    .with_param("host", json!("localhost"))
    .with_param("port", json!(3306))
    .with_param("database", json!("mydb"))
    .with_param("user", json!("user"))
    .with_param("password", json!("pass"));
```

### SQLite Connector

For SQLite database operations

```rust
let config = ConnectorConfig::new("db", "sqlite")
    .with_param("path", json!("/path/to/database.db"));
```

### File System Connector

For file operations with security constraints

```rust
let config = ConnectorConfig::new("storage", "filesystem")
    .with_param("base_path", json!("/data"))
    .with_config("allow_read", json!(true))
    .with_config("allow_write", json!(true))
    .with_config("allow_delete", json!(false));
```

**Supported Operations:**
- READ - Read file content
- WRITE - Write file content
- DELETE - Delete file (if enabled)
- LIST - List directory contents
- MKDIR - Create directory

### Outgoing Webhook Connector

For sending webhooks to external endpoints

```rust
let config = ConnectorConfig::new("slack", "outgoing_webhook")
    .with_param("url", json!("https://hooks.slack.com/services/xxx"))
    .with_config("retry_on_failure", json!(true));

let connector = OutgoingWebhookConnector::new(config);
```

### Incoming Webhook Connector

For receiving webhook events

```rust
let config = ConnectorConfig::new("listener", "incoming_webhook")
    .with_param("port", json!(8080))
    .with_param("path", json!("/webhook"));
```

## Authentication Strategies

### API Key

```rust
AuthConfig::api_key("secret-key", "X-API-Key")
```

With prefix:

```rust
AuthConfig::api_key("token123", "Authorization")
    .with_prefix("Bearer ")
```

### OAuth 2.0

```rust
AuthConfig::oauth2("client-id", "client-secret", "https://example.com/token")
    .add_scope("read:data")
    .add_scope("write:data")
```

### JWT

```rust
AuthConfig::jwt("eyJhbGc...")
    .with_expiration(1704067200)  // Unix timestamp
```

### Basic Authentication

```rust
AuthConfig::basic("username", "password")
```

## Resilience Patterns

### Retry Policies

**Exponential Backoff** (recommended):

```rust
use abcdodaf::integration::connector::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::exponential_backoff(
    4,  // max attempts
    Duration::from_millis(100),  // initial delay
    Duration::from_secs(30),  // max delay
);

// Configure on connector
let config = ConnectorConfig::new("api", "rest_api")
    .with_retries(true, 4);
```

**Fixed Delay**:

```rust
let policy = RetryPolicy::fixed_delay(3, Duration::from_secs(5));
```

**Linear Backoff**:

```rust
let policy = RetryPolicy::linear_backoff(
    4,
    Duration::from_secs(1),
    Duration::from_secs(10),
);
```

### Circuit Breaker

Prevents cascading failures by failing fast when a service is unavailable

```rust
use abcdodaf::integration::connector::{CircuitBreaker, CircuitBreakerState};
use std::time::Duration;

let cb = CircuitBreaker::new(5, Duration::from_secs(60));

// Record outcomes
if request_failed {
    cb.record_failure();
} else {
    cb.record_success();
}

// Check state
match cb.state() {
    CircuitBreakerState::Closed => { /* Normal operation */ }
    CircuitBreakerState::Open => { /* Fail fast */ }
    CircuitBreakerState::HalfOpen => { /* Testing recovery */ }
}
```

**Configuration**:

```rust
let config = ConnectorConfig::new("api", "rest_api")
    .with_circuit_breaker(
        true,       // enable
        5,          // failure threshold
        60          // timeout in seconds
    );
```

## Connection Pooling

For efficient resource management with databases:

```rust
let config = ConnectorConfig::new("db", "postgresql")
    .with_pooling(true, 20);  // Enable pooling with 20 connections
```

## Health Checks

Monitor connector health:

```rust
// Health check single connector
let health = connector.health_check().await?;
match health {
    HealthStatus::Healthy => println!("OK"),
    HealthStatus::Degraded(msg) => println!("Degraded: {}", msg),
    HealthStatus::Unhealthy(msg) => println!("Error: {}", msg),
}

// Health check all connectors
let results = registry.health_check_all().await;
for (name, status, result) in results {
    println!("{}: {:?}", name, status);
}
```

## Custom Connectors

Implement the `Connector` trait to create custom connectors:

```rust
use abcdodaf::integration::connector::{
    Connector, ConnectorConfig, ConnectorRequest, ConnectorResponse,
    ConnectionStatus, HealthStatus, ConnectorResult,
};
use async_trait::async_trait;

pub struct CustomConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
}

#[async_trait]
impl Connector for CustomConnector {
    fn connector_type(&self) -> &str {
        "custom"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Initialize connector
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        // Execute request
        Ok(ConnectorResponse::new(&request.id, 200))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
}
```

Register your custom connector:

```rust
struct CustomFactory;

#[async_trait]
impl ConnectorFactory for CustomFactory {
    async fn create(&self, config: ConnectorConfig)
        -> ConnectorResult<Box<dyn Connector>>
    {
        Ok(Box::new(CustomConnector::new(config)))
    }
}

registry.register_factory("custom", Arc::new(CustomFactory)).await;
```

## Error Handling

Connectors provide detailed error information:

```rust
use abcdodaf::integration::connector::ConnectorError;

match connector.execute(request).await {
    Ok(response) => {
        if response.is_success() {
            // Handle success
        } else {
            // Handle HTTP error
        }
    }
    Err(ConnectorError::Timeout { timeout_secs }) => {
        println!("Request timed out after {}s", timeout_secs);
    }
    Err(ConnectorError::ConnectionError(msg)) => {
        println!("Connection failed: {}", msg);
    }
    Err(ConnectorError::CircuitBreakerOpen) => {
        println!("Service temporarily unavailable");
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

Error classification:

```rust
let is_retryable = error.is_retryable();
let is_rate_limit = error.is_rate_limit();
let is_auth_error = error.is_auth_error();
```

## Integration with Workflows

Use connectors within BPMN workflows:

```rust
use abcdodaf::bpmn::{ProcessBuilder, ProcessExecutor};

let workflow = ProcessBuilder::new("data_sync")
    .add_element("fetch_data", BpmnElement::ServiceTask)
    .add_element("process_data", BpmnElement::ServiceTask)
    .add_element("store_data", BpmnElement::ServiceTask)
    .build()?;

let executor = ProcessExecutor::new();

// Pass connector registry to executor
let result = executor.execute(workflow, Some(registry)).await?;
```

## Configuration Best Practices

1. **Separate Secrets** - Never hardcode credentials in configs
   ```rust
   let token = std::env::var("API_TOKEN")?;
   let auth = AuthConfig::api_key(token, "Authorization");
   ```

2. **Set Appropriate Timeouts**
   ```rust
   let config = ConnectorConfig::new("api", "rest_api")
       .with_timeout(30);  // 30 seconds
   ```

3. **Enable Resilience Features**
   ```rust
   let config = ConnectorConfig::new("api", "rest_api")
       .with_retries(true, 3)
       .with_circuit_breaker(true, 5, 60);
   ```

4. **Use Connection Pooling for Databases**
   ```rust
   let config = ConnectorConfig::new("db", "postgresql")
       .with_pooling(true, 20);
   ```

5. **Validate Paths for File Systems**
   ```rust
   let config = ConnectorConfig::new("fs", "filesystem")
       .with_param("base_path", json!("/safe/directory"))
       .with_config("allow_delete", json!(false));  // Restrict dangerous ops
   ```

## Testing

Use the `MockConnector` for testing:

```rust
#[cfg(test)]
mod tests {
    use abcdodaf::integration::connector::Connector;

    #[tokio::test]
    async fn test_workflow_with_connector() {
        let registry = ConnectorRegistry::new();
        registry.register_factory("mock", Arc::new(MockFactory)).await;

        // Test your workflow
        let config = ConnectorConfig::new("test", "mock");
        let connector = registry.create_connector(config).await?;

        // Assert connector behavior
        assert_eq!(connector.status(), ConnectionStatus::Connected);
    }
}
```

## Performance Tuning

### Request Timeouts
```rust
.with_timeout(30)  // Seconds
```

### Pool Size
```rust
.with_pooling(true, 50)  // For high concurrency
```

### Retry Strategy
```rust
.with_retries(true, 5)  // Up to 5 attempts
```

### Circuit Breaker Sensitivity
```rust
.with_circuit_breaker(true, 10, 120)  // More tolerant
```

## Logging and Monitoring

Enable logging for connectors:

```rust
let config = ConnectorConfig::new("api", "rest_api")
    // logging enabled by default
```

Access connector statistics:

```rust
let stats = connector.statistics();
println!("Requests: {}", stats.total_requests);
println!("Success rate: {}%", stats.successful_requests * 100 / stats.total_requests);
println!("Avg response time: {}ms", stats.avg_response_time_ms);
```

## Examples

See `examples/integration_example.rs` for a complete working example demonstrating:
- Connector creation and configuration
- Multiple connector types
- Error handling
- Health checks
- Request/response execution

## Future Enhancements

Planned features:
1. Message Queue Integration (RabbitMQ, NATS, Kafka)
2. GraphQL Support
3. gRPC Connectors
4. Caching Layer
5. Request/Response Transformation
6. Rate Limiting
7. Request Signing (AWS Signature V4, etc.)
8. WebSocket Support
9. Streaming Response Handling
10. Advanced Logging and Tracing

## Troubleshooting

### Circuit Breaker Stuck Open
The circuit breaker opens after a threshold of failures and will attempt to recover after the timeout period. Check service health and recent errors.

### Connection Pooling Issues
Ensure the database is accessible and credentials are correct. Check pool size matches your concurrency needs.

### Authentication Failures
Verify credentials are correct, tokens haven't expired, and the authentication method matches the API's requirements.

### Timeout Errors
Increase timeout value, check network connectivity, or investigate slow API responses.
