//! Integration tests for the connector framework

#[cfg(test)]
mod tests {
    use abcdodaf::integration::connector::{
        Connector, ConnectorConfig, ConnectorRegistry, ConnectorRequest, ConnectionStatus,
        HealthStatus, AuthConfig,
    };
    use abcdodaf::integration::connectors::{RestApiConnector, PostgresConnector, MySqlConnector};
    use abcdodaf::integration::connector::registry::ConnectorFactory;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;

    struct MockRestApiFactory;

    #[async_trait]
    impl ConnectorFactory for MockRestApiFactory {
        async fn create(
            &self,
            config: ConnectorConfig,
        ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
            Ok(Box::new(RestApiConnector::new(config)))
        }
    }

    struct MockPostgresFactory;

    #[async_trait]
    impl ConnectorFactory for MockPostgresFactory {
        async fn create(
            &self,
            config: ConnectorConfig,
        ) -> abcdodaf::integration::connector::ConnectorResult<Box<dyn Connector>> {
            Ok(Box::new(PostgresConnector::new(config)))
        }
    }

    #[tokio::test]
    async fn test_rest_api_connector_initialization() {
        let config = ConnectorConfig::new("test_api", "rest_api")
            .with_param("url", json!("https://api.example.com/v1"));

        let mut connector = RestApiConnector::new(config);
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);

        connector.initialize().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Connected);

        connector.close().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Closed);
    }

    #[tokio::test]
    async fn test_rest_api_connector_with_auth() {
        let config = ConnectorConfig::new("api_with_auth", "rest_api")
            .with_param("url", json!("https://api.example.com/v1"))
            .with_auth(AuthConfig::api_key("secret-key", "X-API-Key"));

        let mut connector = RestApiConnector::new(config);
        connector.initialize().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_postgres_connector_initialization() {
        let config = ConnectorConfig::new("test_postgres", "postgresql")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(5432))
            .with_param("database", json!("testdb"))
            .with_param("user", json!("testuser"))
            .with_param("password", json!("testpass"));

        let mut connector = PostgresConnector::new(config);
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);

        connector.initialize().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Connected);

        connector.close().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Closed);
    }

    #[tokio::test]
    async fn test_mysql_connector_initialization() {
        let config = ConnectorConfig::new("test_mysql", "mysql")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(3306))
            .with_param("database", json!("testdb"))
            .with_param("user", json!("testuser"))
            .with_param("password", json!("testpass"));

        let mut connector = MySqlConnector::new(config);
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);

        connector.initialize().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Connected);

        connector.close().await.unwrap();
        assert_eq!(connector.status(), ConnectionStatus::Closed);
    }

    #[tokio::test]
    async fn test_connector_registry() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("rest_api", Arc::new(MockRestApiFactory))
            .await;
        registry
            .register_factory("postgresql", Arc::new(MockPostgresFactory))
            .await;

        assert_eq!(registry.factory_count().await, 2);
    }

    #[tokio::test]
    async fn test_connector_registry_create_and_retrieve() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("rest_api", Arc::new(MockRestApiFactory))
            .await;

        let config = ConnectorConfig::new("my_api", "rest_api")
            .with_param("url", json!("https://example.com"));

        let _connector = registry.create_connector(config).await.unwrap();

        assert!(registry.has_connector("my_api").await);
        assert_eq!(registry.connector_count().await, 1);

        let connectors = registry.list_connectors().await;
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0], "my_api");
    }

    #[tokio::test]
    async fn test_connector_registry_remove_connector() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("rest_api", Arc::new(MockRestApiFactory))
            .await;

        let config = ConnectorConfig::new("temp_api", "rest_api")
            .with_param("url", json!("https://example.com"));

        registry.create_connector(config).await.unwrap();
        assert_eq!(registry.connector_count().await, 1);

        registry.remove_connector("temp_api").await.unwrap();
        assert_eq!(registry.connector_count().await, 0);
    }

    #[tokio::test]
    async fn test_connector_health_check() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("rest_api", Arc::new(MockRestApiFactory))
            .await;

        let config = ConnectorConfig::new("health_api", "rest_api")
            .with_param("url", json!("https://example.com"));

        registry.create_connector(config).await.unwrap();

        let health_results = registry.health_check_all().await;
        assert!(!health_results.is_empty());
    }

    #[tokio::test]
    async fn test_connector_request_response() {
        let request = ConnectorRequest::new("GET")
            .with_param("id", json!("123"))
            .with_timeout(30)
            .with_header("X-Custom-Header", "value");

        assert_eq!(request.operation, "GET");
        assert_eq!(request.timeout_secs, Some(30));
        assert_eq!(request.parameters.get("id"), Some(&json!("123")));
        assert_eq!(request.headers.get("X-Custom-Header"), Some(&"value".to_string()));
    }

    #[test]
    fn test_connector_config_defaults() {
        let config = ConnectorConfig::new("test", "rest_api");

        assert_eq!(config.get_timeout().as_secs(), 30);
        assert_eq!(config.get_pool_size(), 10);
        assert_eq!(config.get_max_retries(), 3);
        assert_eq!(config.get_circuit_breaker_threshold(), 5);
    }

    #[test]
    fn test_connector_config_customization() {
        let config = ConnectorConfig::new("test", "rest_api")
            .with_timeout(60)
            .with_pooling(true, 20)
            .with_retries(true, 5)
            .with_circuit_breaker(true, 10, 120);

        assert_eq!(config.get_timeout().as_secs(), 60);
        assert_eq!(config.get_pool_size(), 20);
        assert_eq!(config.get_max_retries(), 5);
        assert_eq!(config.get_circuit_breaker_threshold(), 10);
        assert_eq!(config.get_circuit_breaker_timeout().as_secs(), 120);
    }

    #[test]
    fn test_auth_config_serialization() {
        let api_key_config = AuthConfig::api_key("key123", "X-API-Key");
        let json = serde_json::to_value(&api_key_config).unwrap();
        assert_eq!(json["type"], "api_key");

        let jwt_config = AuthConfig::jwt("token123");
        let json = serde_json::to_value(&jwt_config).unwrap();
        assert_eq!(json["type"], "jwt");

        let basic_config = AuthConfig::basic("user", "pass");
        let json = serde_json::to_value(&basic_config).unwrap();
        assert_eq!(json["type"], "basic");
    }

    #[tokio::test]
    async fn test_multiple_connectors_in_registry() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("rest_api", Arc::new(MockRestApiFactory))
            .await;
        registry
            .register_factory("postgresql", Arc::new(MockPostgresFactory))
            .await;

        // Create REST API connector
        let rest_config = ConnectorConfig::new("api1", "rest_api")
            .with_param("url", json!("https://api.example.com"));
        registry.create_connector(rest_config).await.unwrap();

        // Create PostgreSQL connector
        let pg_config = ConnectorConfig::new("db1", "postgresql")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(5432))
            .with_param("database", json!("testdb"))
            .with_param("user", json!("testuser"))
            .with_param("password", json!("testpass"));
        registry.create_connector(pg_config).await.unwrap();

        assert_eq!(registry.connector_count().await, 2);

        let connectors = registry.list_connectors().await;
        assert!(connectors.contains(&"api1".to_string()));
        assert!(connectors.contains(&"db1".to_string()));
    }

    #[test]
    fn test_retry_policy() {
        use abcdodaf::integration::connector::RetryPolicy;
        use std::time::Duration;

        let policy = RetryPolicy::exponential_backoff(
            4,
            Duration::from_millis(100),
            Duration::from_secs(10),
        );

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(policy.should_retry(3));
        assert!(!policy.should_retry(4));

        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);
        assert!(delay2.as_millis() >= delay1.as_millis());
    }

    #[test]
    fn test_circuit_breaker() {
        use abcdodaf::integration::connector::CircuitBreaker;
        use std::time::Duration;
        use std::thread;

        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        assert_eq!(cb.state(), ConnectionStatus::Closed);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), ConnectionStatus::Open);

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), ConnectionStatus::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), ConnectionStatus::Closed);
    }
}
