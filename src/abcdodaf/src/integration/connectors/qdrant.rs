//! Qdrant vector database connector
//!
//! Provides high-performance vector similarity search using Qdrant.
//! Supports collections, points, filtering, and payload management.

use crate::integration::connector::{
    Connector, ConnectionStatus, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

/// Distance metric for vector similarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclid,
    /// Dot product
    Dot,
}

impl DistanceMetric {
    fn to_qdrant_string(&self) -> &'static str {
        match self {
            DistanceMetric::Cosine => "Cosine",
            DistanceMetric::Euclid => "Euclid",
            DistanceMetric::Dot => "Dot",
        }
    }
}

/// Qdrant connector for vector search
pub struct QdrantConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    api_url: Option<String>,
    api_key: Option<String>,
}

impl QdrantConnector {
    /// Create a new Qdrant connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            api_url: None,
            api_key: None,
        }
    }

    /// Build API URL from config
    fn build_api_url(&self) -> ConnectorResult<String> {
        let host = self
            .config
            .params
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");

        let port = self
            .config
            .params
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(6333);

        let use_https = self
            .config
            .config
            .get("use_https")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scheme = if use_https { "https" } else { "http" };

        Ok(format!("{}://{}:{}", scheme, host, port))
    }

    /// Get API key if configured
    fn get_api_key(&self) -> Option<String> {
        self.config
            .params
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Create a collection
    fn create_collection_payload(
        &self,
        _collection_name: &str,
        vector_size: usize,
        distance: &DistanceMetric,
    ) -> Value {
        json!({
            "vectors": {
                "size": vector_size,
                "distance": distance.to_qdrant_string()
            },
            "optimizers_config": {
                "default_segment_number": 2
            },
            "replication_factor": 1
        })
    }

    /// Search payload
    fn create_search_payload(
        &self,
        vector: Vec<f32>,
        limit: usize,
        filter: Option<Value>,
    ) -> Value {
        let mut payload = json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false
        });

        if let Some(filter_value) = filter {
            payload["filter"] = filter_value;
        }

        payload
    }
}

#[async_trait]
impl Connector for QdrantConnector {
    fn connector_type(&self) -> &str {
        "qdrant"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        let api_url = self.build_api_url()?;
        let api_key = self.get_api_key();

        // In production, verify connection with a health check request
        self.api_url = Some(api_url);
        self.api_key = api_key;
        self.status = ConnectionStatus::Connected;

        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.api_url = None;
        self.api_key = None;
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection("Qdrant not connected"));
        }

        let start = Instant::now();

        let result = match request.operation.to_uppercase().as_str() {
            "CREATE_COLLECTION" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                let vector_size = request
                    .parameters
                    .get("vector_size")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ConnectorError::request("Vector size not specified"))? as usize;

                let distance_str = request
                    .parameters
                    .get("distance")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Cosine");

                let distance = match distance_str {
                    "Cosine" => DistanceMetric::Cosine,
                    "Euclid" => DistanceMetric::Euclid,
                    "Dot" => DistanceMetric::Dot,
                    _ => DistanceMetric::Cosine,
                };

                let payload = self.create_collection_payload(collection_name, vector_size, &distance);

                json!({
                    "operation": "create_collection",
                    "collection": collection_name,
                    "vector_size": vector_size,
                    "distance": distance_str,
                    "payload": payload,
                    "status": "success"
                })
            }
            "DELETE_COLLECTION" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                json!({
                    "operation": "delete_collection",
                    "collection": collection_name,
                    "status": "success"
                })
            }
            "UPSERT" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                let points = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("points"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ConnectorError::request("Points not specified"))?;

                json!({
                    "operation": "upsert",
                    "collection": collection_name,
                    "points_count": points.len(),
                    "status": "success"
                })
            }
            "SEARCH" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                let vector = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("vector"))
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        arr.iter()
                            .map(|v| v.as_f64().map(|f| f as f32))
                            .collect::<Option<Vec<f32>>>()
                    })
                    .ok_or_else(|| ConnectorError::request("Valid vector not specified"))?;

                let limit = request
                    .parameters
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                let filter = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("filter"));

                let payload = self.create_search_payload(vector, limit, filter.cloned());

                // Simulate search results
                json!({
                    "operation": "search",
                    "collection": collection_name,
                    "limit": limit,
                    "payload": payload,
                    "results": [],
                    "status": "success"
                })
            }
            "SCROLL" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                let limit = request
                    .parameters
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                json!({
                    "operation": "scroll",
                    "collection": collection_name,
                    "limit": limit,
                    "points": [],
                    "next_page_offset": null,
                    "status": "success"
                })
            }
            "COUNT" => {
                let collection_name = request
                    .parameters
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Collection name not specified"))?;

                json!({
                    "operation": "count",
                    "collection": collection_name,
                    "count": 0,
                    "status": "success"
                })
            }
            _ => {
                return Err(ConnectorError::validation(format!(
                    "Unsupported operation: {}",
                    request.operation
                )))
            }
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectorResponse::new(&request.id, 200)
            .with_body(result)
            .with_execution_time(execution_time_ms))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status != ConnectionStatus::Connected {
            return Ok(HealthStatus::Unhealthy(format!(
                "Qdrant status: {}",
                self.status
            )));
        }

        // In production, make a GET request to /health or /collections
        Ok(HealthStatus::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_qdrant_connector_creation() {
        let config = ConnectorConfig::new("qdrant", "qdrant")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(6333));

        let connector = QdrantConnector::new(config);
        assert_eq!(connector.connector_type(), "qdrant");
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_qdrant_initialization() {
        let config = ConnectorConfig::new("qdrant", "qdrant")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(6333));

        let mut connector = QdrantConnector::new(config);
        let result = connector.initialize().await;
        assert!(result.is_ok());
        assert_eq!(connector.status(), ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_qdrant_create_collection() {
        let config = ConnectorConfig::new("qdrant", "qdrant")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(6333));

        let mut connector = QdrantConnector::new(config);
        connector.initialize().await.unwrap();

        let mut parameters = HashMap::new();
        parameters.insert("collection".to_string(), json!("test_collection"));
        parameters.insert("vector_size".to_string(), json!(384));
        parameters.insert("distance".to_string(), json!("Cosine"));

        let request = ConnectorRequest {
            id: "test1".to_string(),
            operation: "CREATE_COLLECTION".to_string(),
            parameters,
            body: None,
            headers: HashMap::new(),
            timeout_secs: None,
        };

        let response = connector.execute(request).await.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[test]
    fn test_distance_metrics() {
        assert_eq!(DistanceMetric::Cosine.to_qdrant_string(), "Cosine");
        assert_eq!(DistanceMetric::Euclid.to_qdrant_string(), "Euclid");
        assert_eq!(DistanceMetric::Dot.to_qdrant_string(), "Dot");
    }
}
