# ABCDODAF Integration Layer - Quick Reference

## File Locations

### Core Framework
| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Connector Trait | `src/integration/connector/mod.rs` | 286 | Base interface for all connectors |
| Error Types | `src/integration/connector/error.rs` | 162 | Error handling and classification |
| Authentication | `src/integration/connector/auth.rs` | 351 | Auth strategies (API Key, OAuth2, JWT, Basic) |
| Configuration | `src/integration/connector/config.rs` | 315 | Config builder and context |
| Retry Policies | `src/integration/connector/retry.rs` | 203 | Exponential, linear, fixed backoff |
| Circuit Breaker | `src/integration/connector/circuit_breaker.rs` | 305 | Fault tolerance pattern |
| Registry | `src/integration/connector/registry.rs` | 261 | Connector management and factory |

### Concrete Connectors
| Connector | File | Lines | Purpose |
|-----------|------|-------|---------|
| REST API | `src/integration/connectors/rest_api.rs` | 325 | HTTP-based integrations |
| Databases | `src/integration/connectors/database.rs` | 494 | PostgreSQL, MySQL, SQLite |
| Webhooks | `src/integration/connectors/webhook.rs` | 348 | Incoming and outgoing webhooks |
| File System | `src/integration/connectors/filesystem.rs` | 372 | Safe file operations |

### Documentation
| Document | File | Lines | Purpose |
|----------|------|-------|---------|
| Integration Guide | `INTEGRATION_GUIDE.md` | 583 | Complete user guide |
| Task Summary | `TASK_9_SUMMARY.md` | 281 | Implementation summary |
| Implementation Notes | `IMPLEMENTATION_NOTES.md` | 630 | Design decisions and architecture |
| Quick Reference | `QUICK_REFERENCE.md` | - | This file |
| Completion Checklist | `TASK_9_COMPLETION_CHECKLIST.md` | 425 | Requirements verification |

### Examples & Tests
| File | Lines | Purpose |
|------|-------|---------|
| `examples/integration_example.rs` | 253 | Comprehensive working example |
| `tests/integration_connector_tests.rs` | 295 | Integration test suite |

## Quick Start Snippets

### Create and Use a REST API Connector
```rust
use abcdodaf::integration::{ConnectorConfig, AuthConfig};
use serde_json::json;

// Configure
let config = ConnectorConfig::new("github", "rest_api")
    .with_param("url", json!("https://api.github.com"))
    .with_auth(AuthConfig::api_key("token", "Authorization").with_prefix("Bearer "))
    .with_timeout(30);

// Create
let registry = ConnectorRegistry::new();
registry.register_factory("rest_api", Arc::new(RestApiFactory)).await;
let connector = registry.create_connector(config).await?;

// Use
let request = ConnectorRequest::new("GET")
    .with_param("page", json!(1));
let response = connector.read().await.execute(request).await?;
```

### Create and Use a Database Connector
```rust
// PostgreSQL
let config = ConnectorConfig::new("mydb", "postgresql")
    .with_param("host", json!("localhost"))
    .with_param("database", json!("myapp"))
    .with_param("user", json!("user"))
    .with_param("password", json!("pass"));

// MySQL
let config = ConnectorConfig::new("mydb", "mysql")
    .with_param("host", json!("localhost"))
    .with_param("port", json!(3306))
    .with_param("database", json!("myapp"))
    .with_param("user", json!("user"))
    .with_param("password", json!("pass"));

// SQLite
let config = ConnectorConfig::new("mydb", "sqlite")
    .with_param("path", json!("/path/to/db.sqlite"));
```

### Create and Use a File System Connector
```rust
let config = ConnectorConfig::new("storage", "filesystem")
    .with_param("base_path", json!("/safe/directory"))
    .with_config("allow_read", json!(true))
    .with_config("allow_write", json!(true))
    .with_config("allow_delete", json!(false));
```

### Enable Resilience
```rust
let config = ConnectorConfig::new("api", "rest_api")
    .with_param("url", json!("https://api.example.com"))
    // Retry with exponential backoff
    .with_retries(true, 3)
    // Circuit breaker: open after 5 failures, retry after 60s
    .with_circuit_breaker(true, 5, 60)
    // Connection pooling
    .with_pooling(true, 20);
```

### Handle Errors
```rust
match connector.execute(request).await {
    Ok(response) if response.is_success() => { /* OK */ }
    Ok(response) => { /* HTTP error */ }
    Err(ConnectorError::Timeout { .. }) => { /* Timeout */ }
    Err(ConnectorError::CircuitBreakerOpen) => { /* Unavailable */ }
    Err(e) if e.is_retryable() => { /* Retry */ }
    Err(e) => { /* Handle error */ }
}
```

### Check Health
```rust
let status = connector.read().await.health_check().await?;
match status {
    HealthStatus::Healthy => { /* OK */ }
    HealthStatus::Degraded(msg) => { /* Warn */ }
    HealthStatus::Unhealthy(msg) => { /* Error */ }
}
```

## API Reference

### Core Traits
- `Connector` - Main trait for all connectors
- `AuthStrategy` - Authentication interface
- `ConnectorFactory` - Factory for creating connectors

### Key Structures
- `ConnectorConfig` - Configuration builder
- `ConnectorContext` - Execution context
- `ConnectorRequest` - Request to connector
- `ConnectorResponse` - Response from connector
- `ConnectorRegistry` - Connector management

### Configuration
- `AuthConfig` - Authentication enum (ApiKey, OAuth2, Jwt, Basic)
- `RetryPolicy` - Retry configuration
- `CircuitBreaker` - Fault tolerance

### Enums
- `ConnectionStatus` - Disconnected, Connecting, Connected, Reconnecting, Error, Closed
- `HealthStatus` - Healthy, Degraded, Unhealthy
- `CircuitBreakerState` - Closed, Open, HalfOpen

### Connector Types
- `RestApiConnector` - HTTP integrations
- `PostgresConnector` - PostgreSQL database
- `MySqlConnector` - MySQL/MariaDB database
- `SqliteConnector` - SQLite database
- `OutgoingWebhookConnector` - Send webhooks
- `IncomingWebhookConnector` - Receive webhooks
- `FileSystemConnector` - File operations

## Common Operations

### List All Connectors
```rust
let connectors = registry.list_connectors().await;
for name in connectors {
    println!("{}", name);
}
```

### Check All Health
```rust
let results = registry.health_check_all().await;
for (name, status, result) in results {
    println!("{}: {:?}", name, status);
}
```

### Remove Connector
```rust
registry.remove_connector("api").await?;
```

### Close All
```rust
registry.close_all().await?;
```

## Retry Strategies

### Exponential Backoff (Recommended)
```rust
RetryPolicy::exponential_backoff(
    4,  // max attempts
    Duration::from_millis(100),  // initial delay
    Duration::from_secs(30),  // max delay
)
```

### Fixed Delay
```rust
RetryPolicy::fixed_delay(
    3,  // attempts
    Duration::from_secs(5)  // delay between retries
)
```

### Linear Backoff
```rust
RetryPolicy::linear_backoff(
    4,  // attempts
    Duration::from_secs(1),  // initial delay
    Duration::from_secs(10)  // max delay
)
```

## Error Types

| Error | Meaning |
|-------|---------|
| ConfigError | Invalid configuration |
| ConnectionError | Cannot connect |
| RequestError | Request execution failed |
| ResponseError | Response processing failed |
| Timeout | Request timed out |
| AuthError | Authentication failed |
| ValidationError | Data validation failed |
| NotFound | Resource not found |
| RateLimit | Rate limit exceeded |
| CircuitBreakerOpen | Service unavailable (CB) |
| MaxRetriesExceeded | Gave up after retries |

## Authentication Examples

### API Key
```rust
AuthConfig::api_key("secret", "X-API-Key")
```

### API Key with Prefix
```rust
AuthConfig::api_key("token", "Authorization").with_prefix("Bearer ")
```

### OAuth 2.0
```rust
AuthConfig::oauth2("client-id", "client-secret", "https://example.com/token")
    .add_scope("read:data")
```

### JWT
```rust
AuthConfig::jwt("eyJhbGc...")
```

### Basic Auth
```rust
AuthConfig::basic("username", "password")
```

## Best Practices

1. **Use environment variables for secrets**
   ```rust
   let token = std::env::var("API_TOKEN")?;
   ```

2. **Enable resilience**
   ```rust
   .with_retries(true, 3)
   .with_circuit_breaker(true, 5, 60)
   ```

3. **Set appropriate timeouts**
   ```rust
   .with_timeout(30)  // 30 seconds
   ```

4. **Use connection pooling for databases**
   ```rust
   .with_pooling(true, 20)
   ```

5. **Validate file paths**
   ```rust
   .with_param("base_path", json!("/safe/directory"))
   ```

## Performance Tips

- Use connection pooling for databases
- Set reasonable timeout values
- Configure circuit breaker to fail fast
- Use exponential backoff for retries
- Enable health checks

## Testing

Run the example:
```bash
cargo run --example integration_example
```

Run tests:
```bash
cargo test --test integration_connector_tests
```

## Module Structure

```
integration/
├── connector/
│   ├── mod.rs          # Core traits
│   ├── error.rs        # Error handling
│   ├── auth.rs         # Authentication
│   ├── config.rs       # Configuration
│   ├── retry.rs        # Retry policies
│   ├── circuit_breaker.rs  # Circuit breaker
│   └── registry.rs     # Registry
└── connectors/
    ├── mod.rs
    ├── rest_api.rs     # REST connectors
    ├── database.rs     # DB connectors
    ├── webhook.rs      # Webhook connectors
    └── filesystem.rs   # File system connector
```

## Further Reading

- `INTEGRATION_GUIDE.md` - Complete guide with examples
- `IMPLEMENTATION_NOTES.md` - Design decisions and architecture
- `examples/integration_example.rs` - Working example code
- `tests/integration_connector_tests.rs` - Test examples

## Support

For issues and questions:
1. Check `INTEGRATION_GUIDE.md` troubleshooting section
2. Review `examples/integration_example.rs` for usage patterns
3. Check `tests/integration_connector_tests.rs` for test examples
4. Review error message using `error.is_retryable()`, etc.
