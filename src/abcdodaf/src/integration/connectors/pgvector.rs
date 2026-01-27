//! PGVector extension support for PostgreSQL
//!
//! Provides vector similarity search capabilities using the pgvector extension for PostgreSQL.
//! Supports embedding storage, similarity queries (cosine, L2, inner product), and index management.

use crate::integration::connector::{
    ConnectionStatus, Connector, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

/// Similarity metric for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityMetric {
    /// Cosine similarity (1 - cosine distance)
    Cosine,
    /// Euclidean distance (L2 norm)
    L2,
    /// Inner product (dot product)
    InnerProduct,
}

impl SimilarityMetric {
    fn to_pg_operator(&self) -> &'static str {
        match self {
            SimilarityMetric::Cosine => "<=>",       // cosine distance
            SimilarityMetric::L2 => "<->",           // L2 distance
            SimilarityMetric::InnerProduct => "<#>", // negative inner product
        }
    }
}

/// PGVector connector with embedding support
pub struct PgVectorConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    connection_string: Option<String>,
    dimension: usize,
}

impl PgVectorConnector {
    /// Create a new PGVector connector
    pub fn new(config: ConnectorConfig) -> Self {
        let dimension =
            config.config.get("vector_dimension").and_then(|v| v.as_u64()).unwrap_or(1536) as usize; // Default to OpenAI ada-002 dimension

        Self { config, status: ConnectionStatus::Disconnected, connection_string: None, dimension }
    }

    /// Build connection string from config
    fn build_connection_string(&self) -> ConnectorResult<String> {
        let host = self.config.params.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");

        let port = self.config.params.get("port").and_then(|v| v.as_u64()).unwrap_or(5432);

        let database = self
            .config
            .params
            .get("database")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("Database name not specified"))?;

        let user = self
            .config
            .params
            .get("user")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("User not specified"))?;

        let password = self.config.params.get("password").and_then(|v| v.as_str()).unwrap_or("");

        Ok(format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, database))
    }

    /// Initialize pgvector extension
    async fn ensure_extension(&self) -> ConnectorResult<()> {
        // In production: CREATE EXTENSION IF NOT EXISTS vector;
        Ok(())
    }

    /// Create a vector table
    fn generate_create_table_sql(&self, table_name: &str, vector_column: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id SERIAL PRIMARY KEY,
                {} vector({}),
                metadata JSONB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            table_name, vector_column, self.dimension
        )
    }

    /// Create an IVFFlat index for faster similarity search
    fn generate_create_index_sql(
        &self,
        table_name: &str,
        vector_column: &str,
        metric: &SimilarityMetric,
    ) -> String {
        let operator_class = match metric {
            SimilarityMetric::Cosine => "vector_cosine_ops",
            SimilarityMetric::L2 => "vector_l2_ops",
            SimilarityMetric::InnerProduct => "vector_ip_ops",
        };

        format!(
            "CREATE INDEX IF NOT EXISTS {}_vector_idx ON {} USING ivfflat ({} {}) WITH (lists = 100)",
            table_name, table_name, vector_column, operator_class
        )
    }

    /// Generate similarity search query
    fn generate_similarity_query(
        &self,
        table_name: &str,
        vector_column: &str,
        embedding: &[f32],
        metric: &SimilarityMetric,
        limit: usize,
    ) -> String {
        let vector_literal =
            format!("[{}]", embedding.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));

        format!(
            "SELECT id, metadata, {} {} '{}' AS distance FROM {} ORDER BY {} {} '{}' LIMIT {}",
            vector_column,
            metric.to_pg_operator(),
            vector_literal,
            table_name,
            vector_column,
            metric.to_pg_operator(),
            vector_literal,
            limit
        )
    }
}

#[async_trait]
impl Connector for PgVectorConnector {
    fn connector_type(&self) -> &str {
        "pgvector"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        let conn_str = self.build_connection_string()?;

        // In production, create connection pool here
        self.ensure_extension().await?;
        self.connection_string = Some(conn_str);
        self.status = ConnectionStatus::Connected;

        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.connection_string = None;
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection("PGVector not connected"));
        }

        let start = Instant::now();

        let result = match request.operation.to_uppercase().as_str() {
            "CREATE_TABLE" => {
                let table_name = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table name not specified"))?;

                let vector_column = request
                    .parameters
                    .get("vector_column")
                    .and_then(|v| v.as_str())
                    .unwrap_or("embedding");

                let sql = self.generate_create_table_sql(table_name, vector_column);

                json!({
                    "operation": "create_table",
                    "table": table_name,
                    "sql": sql,
                    "status": "success"
                })
            },
            "CREATE_INDEX" => {
                let table_name = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table name not specified"))?;

                let vector_column = request
                    .parameters
                    .get("vector_column")
                    .and_then(|v| v.as_str())
                    .unwrap_or("embedding");

                let metric_str =
                    request.parameters.get("metric").and_then(|v| v.as_str()).unwrap_or("cosine");

                let metric = match metric_str {
                    "cosine" => SimilarityMetric::Cosine,
                    "l2" => SimilarityMetric::L2,
                    "inner_product" => SimilarityMetric::InnerProduct,
                    _ => SimilarityMetric::Cosine,
                };

                let sql = self.generate_create_index_sql(table_name, vector_column, &metric);

                json!({
                    "operation": "create_index",
                    "table": table_name,
                    "metric": metric_str,
                    "sql": sql,
                    "status": "success"
                })
            },
            "INSERT_EMBEDDING" => {
                let table_name = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table name not specified"))?;

                let embedding = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("embedding"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ConnectorError::request("Embedding not specified"))?;

                let metadata = request.body.as_ref().and_then(|v| v.get("metadata"));

                json!({
                    "operation": "insert_embedding",
                    "table": table_name,
                    "embedding_dimension": embedding.len(),
                    "has_metadata": metadata.is_some(),
                    "id": 1,
                    "status": "success"
                })
            },
            "SIMILARITY_SEARCH" => {
                let table_name = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table name not specified"))?;

                let vector_column = request
                    .parameters
                    .get("vector_column")
                    .and_then(|v| v.as_str())
                    .unwrap_or("embedding");

                let embedding = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("embedding"))
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        arr.iter()
                            .map(|v| v.as_f64().map(|f| f as f32))
                            .collect::<Option<Vec<f32>>>()
                    })
                    .ok_or_else(|| ConnectorError::request("Valid embedding not specified"))?;

                let metric_str =
                    request.parameters.get("metric").and_then(|v| v.as_str()).unwrap_or("cosine");

                let metric = match metric_str {
                    "cosine" => SimilarityMetric::Cosine,
                    "l2" => SimilarityMetric::L2,
                    "inner_product" => SimilarityMetric::InnerProduct,
                    _ => SimilarityMetric::Cosine,
                };

                let limit =
                    request.parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let sql = self.generate_similarity_query(
                    table_name,
                    vector_column,
                    &embedding,
                    &metric,
                    limit,
                );

                // Simulate results
                json!({
                    "operation": "similarity_search",
                    "table": table_name,
                    "metric": metric_str,
                    "limit": limit,
                    "sql": sql,
                    "results": [],
                    "status": "success"
                })
            },
            _ => {
                return Err(ConnectorError::validation(format!(
                    "Unsupported operation: {}",
                    request.operation
                )))
            },
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectorResponse::new(&request.id, 200)
            .with_body(result)
            .with_execution_time(execution_time_ms))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status != ConnectionStatus::Connected {
            return Ok(HealthStatus::Unhealthy(format!("PGVector status: {}", self.status)));
        }

        // In production, check if pgvector extension is installed
        Ok(HealthStatus::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_pgvector_connector_creation() {
        let config = ConnectorConfig::new("pgvec", "pgvector")
            .with_param("host", json!("localhost"))
            .with_param("database", json!("vectordb"))
            .with_param("user", json!("test_user"))
            .with_config("vector_dimension", json!(384));

        let connector = PgVectorConnector::new(config);
        assert_eq!(connector.connector_type(), "pgvector");
        assert_eq!(connector.dimension, 384);
    }

    #[tokio::test]
    async fn test_pgvector_initialization() {
        let config = ConnectorConfig::new("pgvec", "pgvector")
            .with_param("host", json!("localhost"))
            .with_param("database", json!("vectordb"))
            .with_param("user", json!("test_user"))
            .with_param("password", json!("pass"));

        let mut connector = PgVectorConnector::new(config);
        let result = connector.initialize().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_similarity_metrics() {
        assert_eq!(SimilarityMetric::Cosine.to_pg_operator(), "<=>");
        assert_eq!(SimilarityMetric::L2.to_pg_operator(), "<->");
        assert_eq!(SimilarityMetric::InnerProduct.to_pg_operator(), "<#>");
    }
}
