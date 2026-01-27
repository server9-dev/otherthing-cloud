//! Connector registry for managing multiple connectors

use super::error::{ConnectorError, ConnectorResult};
use super::{Connector, ConnectorConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connector factory trait
#[async_trait::async_trait]
pub trait ConnectorFactory: Send + Sync {
    /// Create a connector from configuration
    async fn create(&self, config: ConnectorConfig) -> ConnectorResult<Box<dyn Connector>>;
}

/// Connector registry for managing multiple connectors
#[derive(Clone)]
pub struct ConnectorRegistry {
    connectors: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn Connector>>>>>>,
    factories: Arc<RwLock<HashMap<String, Arc<dyn ConnectorFactory>>>>,
}

impl ConnectorRegistry {
    /// Create a new connector registry
    pub fn new() -> Self {
        Self {
            connectors: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a connector factory
    pub async fn register_factory(
        &self,
        connector_type: impl Into<String>,
        factory: Arc<dyn ConnectorFactory>,
    ) {
        let mut factories = self.factories.write().await;
        factories.insert(connector_type.into(), factory);
    }

    /// Create and register a connector
    pub async fn create_connector(
        &self,
        config: ConnectorConfig,
    ) -> ConnectorResult<Arc<RwLock<Box<dyn Connector>>>> {
        let factories = self.factories.read().await;

        let factory = factories
            .get(&config.connector_type)
            .ok_or_else(|| {
                ConnectorError::config(format!(
                    "No factory registered for connector type: {}",
                    config.connector_type
                ))
            })?
            .clone();

        drop(factories);

        let mut connector = factory.create(config.clone()).await?;
        connector.initialize().await?;

        let connector = Arc::new(RwLock::new(connector));

        let mut connectors = self.connectors.write().await;
        connectors.insert(config.name.clone(), connector.clone());

        Ok(connector)
    }

    /// Get a registered connector
    pub async fn get_connector(
        &self,
        name: &str,
    ) -> ConnectorResult<Arc<RwLock<Box<dyn Connector>>>> {
        let connectors = self.connectors.read().await;
        connectors
            .get(name)
            .cloned()
            .ok_or_else(|| ConnectorError::request(format!("Connector not found: {}", name)))
    }

    /// Check if connector exists
    pub async fn has_connector(&self, name: &str) -> bool {
        let connectors = self.connectors.read().await;
        connectors.contains_key(name)
    }

    /// List all registered connector names
    pub async fn list_connectors(&self) -> Vec<String> {
        let connectors = self.connectors.read().await;
        connectors.keys().cloned().collect()
    }

    /// Remove a connector
    pub async fn remove_connector(&self, name: &str) -> ConnectorResult<()> {
        let mut connectors = self.connectors.write().await;
        if let Some(connector) = connectors.remove(name) {
            let mut conn = connector.write().await;
            conn.close().await?;
        }
        Ok(())
    }

    /// Close all connectors
    pub async fn close_all(&self) -> ConnectorResult<()> {
        let mut connectors = self.connectors.write().await;

        for (_, connector) in connectors.iter_mut() {
            let mut conn = connector.write().await;
            conn.close().await?;
        }

        connectors.clear();
        Ok(())
    }

    /// Get connector count
    pub async fn connector_count(&self) -> usize {
        let connectors = self.connectors.read().await;
        connectors.len()
    }

    /// Get factory count
    pub async fn factory_count(&self) -> usize {
        let factories = self.factories.read().await;
        factories.len()
    }

    /// Health check all connectors
    pub async fn health_check_all(
        &self,
    ) -> Vec<(String, super::HealthStatus, super::error::ConnectorResult<()>)> {
        let connectors = self.connectors.read().await;
        let mut results = Vec::new();

        for (name, connector) in connectors.iter() {
            let conn = connector.read().await;
            let health = conn.health_check().await;
            let status = match &health {
                Ok(status) => status.clone(),
                Err(_) => super::HealthStatus::Unhealthy("Health check failed".to_string()),
            };
            results.push((name.clone(), status, health.map(|_| ())));
        }

        results
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConnectorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectorRegistry").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::connector::{
        ConnectionStatus, ConnectorResponse, HealthStatus,
    };
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockConnector {
        initialized: bool,
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Connector for MockConnector {
        fn connector_type(&self) -> &str {
            "mock"
        }

        fn name(&self) -> &str {
            "mock_connector"
        }

        fn status(&self) -> ConnectionStatus {
            if self.initialized {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            }
        }

        async fn initialize(&mut self) -> ConnectorResult<()> {
            self.initialized = true;
            Ok(())
        }

        async fn close(&mut self) -> ConnectorResult<()> {
            self.initialized = false;
            Ok(())
        }

        async fn execute(&self, request: crate::integration::ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(ConnectorResponse::new(request.id, 200))
        }

        async fn health_check(&self) -> ConnectorResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    struct MockFactory;

    #[async_trait]
    impl ConnectorFactory for MockFactory {
        async fn create(&self, _config: ConnectorConfig) -> ConnectorResult<Box<dyn Connector>> {
            Ok(Box::new(MockConnector {
                initialized: false,
                call_count: Arc::new(AtomicUsize::new(0)),
            }))
        }
    }

    #[tokio::test]
    async fn test_registry_create_connector() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("mock", Arc::new(MockFactory))
            .await;

        let config = ConnectorConfig::new("test_connector", "mock");
        let connector = registry.create_connector(config).await.unwrap();

        let conn = connector.read().await;
        assert_eq!(conn.status(), ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_registry_list_connectors() {
        let registry = ConnectorRegistry::new();
        registry
            .register_factory("mock", Arc::new(MockFactory))
            .await;

        registry
            .create_connector(ConnectorConfig::new("conn1", "mock"))
            .await
            .unwrap();
        registry
            .create_connector(ConnectorConfig::new("conn2", "mock"))
            .await
            .unwrap();

        let connectors = registry.list_connectors().await;
        assert_eq!(connectors.len(), 2);
    }
}
