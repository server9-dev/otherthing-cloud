//! Database connectors for PostgreSQL, MySQL, and SQLite

use crate::integration::connector::{
    ConnectionStatus, Connector, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus,
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;

/// URL-encode a string for use in connection strings
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// PostgreSQL connector
pub struct PostgresConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
}

impl PostgresConnector {
    /// Create a new PostgreSQL connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config, status: ConnectionStatus::Disconnected }
    }

    /// Get connection string
    ///
    /// SECURITY: Password is retrieved from config params. In production:
    /// - Store passwords in environment variables or secret management system
    /// - Use urlencoding for special characters in passwords
    /// - Never log connection strings containing passwords
    fn get_connection_string(&self) -> ConnectorResult<String> {
        let host = self
            .config
            .params
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("Host not configured"))?;

        let port = self.config.params.get("port").and_then(|v| v.as_u64()).unwrap_or(5432);

        let database = self
            .config
            .params
            .get("database")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("Database not configured"))?;

        let user = self
            .config
            .params
            .get("user")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("User not configured"))?;

        // Try to get password from environment variable first, then fall back to config
        let password =
            std::env::var(format!("POSTGRES_PASSWORD_{}", self.config.name.to_uppercase()))
                .or_else(|_| std::env::var("POSTGRES_PASSWORD"))
                .or_else(|_| {
                    self.config
                        .params
                        .get("password")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .ok_or_else(|| {
                            ConnectorError::config(
                        "Password not configured (set via POSTGRES_PASSWORD env var or config)",
                    )
                        })
                })?;

        // URL-encode password to handle special characters
        let encoded_password = url_encode(&password);

        Ok(format!("postgresql://{}:{}@{}:{}/{}", user, encoded_password, host, port, database))
    }
}

#[async_trait]
impl Connector for PostgresConnector {
    fn connector_type(&self) -> &str {
        "postgresql"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify configuration
        self.get_connection_string()?;

        // In a real implementation, you would create a connection pool here
        // For now, we just verify the config is valid
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection(format!(
                "Database not connected: {}",
                self.status
            )));
        }

        let start = Instant::now();

        // Simulate query execution
        // In a real implementation, this would execute actual SQL queries
        let result = match request.operation.to_uppercase().as_str() {
            "QUERY" => {
                let query = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("query"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Query not specified"))?;

                json!({
                    "query": query,
                    "rows_affected": 0,
                    "results": []
                })
            },
            "INSERT" => {
                let table = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table not specified"))?;

                json!({
                    "operation": "insert",
                    "table": table,
                    "rows_affected": 1
                })
            },
            "UPDATE" => {
                let table = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table not specified"))?;

                json!({
                    "operation": "update",
                    "table": table,
                    "rows_affected": 1
                })
            },
            "DELETE" => {
                let table = request
                    .parameters
                    .get("table")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Table not specified"))?;

                json!({
                    "operation": "delete",
                    "table": table,
                    "rows_affected": 1
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
            return Ok(HealthStatus::Unhealthy(format!("Database status: {}", self.status)));
        }

        // In a real implementation, you would execute a simple query like "SELECT 1"
        Ok(HealthStatus::Healthy)
    }
}

/// MySQL connector
pub struct MySqlConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
}

impl MySqlConnector {
    /// Create a new MySQL connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config, status: ConnectionStatus::Disconnected }
    }

    /// Get connection string
    ///
    /// SECURITY: Password is retrieved from config params. In production:
    /// - Store passwords in environment variables or secret management system
    /// - Use urlencoding for special characters in passwords
    /// - Never log connection strings containing passwords
    fn get_connection_string(&self) -> ConnectorResult<String> {
        let host = self
            .config
            .params
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("Host not configured"))?;

        let port = self.config.params.get("port").and_then(|v| v.as_u64()).unwrap_or(3306);

        let database = self
            .config
            .params
            .get("database")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("Database not configured"))?;

        let user = self
            .config
            .params
            .get("user")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::config("User not configured"))?;

        // Try to get password from environment variable first, then fall back to config
        let password = std::env::var(format!("MYSQL_PASSWORD_{}", self.config.name.to_uppercase()))
            .or_else(|_| std::env::var("MYSQL_PASSWORD"))
            .or_else(|_| {
                self.config
                    .params
                    .get("password")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| {
                        ConnectorError::config(
                            "Password not configured (set via MYSQL_PASSWORD env var or config)",
                        )
                    })
            })?;

        // URL-encode password to handle special characters
        let encoded_password = url_encode(&password);

        Ok(format!("mysql://{}:{}@{}:{}/{}", user, encoded_password, host, port, database))
    }
}

#[async_trait]
impl Connector for MySqlConnector {
    fn connector_type(&self) -> &str {
        "mysql"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify configuration
        self.get_connection_string()?;

        // In a real implementation, you would create a connection pool here
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection(format!(
                "Database not connected: {}",
                self.status
            )));
        }

        let start = Instant::now();

        // Simulate query execution
        let result = match request.operation.to_uppercase().as_str() {
            "QUERY" => {
                let query = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("query"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Query not specified"))?;

                json!({
                    "query": query,
                    "rows_affected": 0,
                    "results": []
                })
            },
            "INSERT" | "UPDATE" | "DELETE" => {
                json!({
                    "operation": request.operation,
                    "rows_affected": 1
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
            return Ok(HealthStatus::Unhealthy(format!("Database status: {}", self.status)));
        }

        Ok(HealthStatus::Healthy)
    }
}

/// SQLite connector
pub struct SqliteConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
}

impl SqliteConnector {
    /// Create a new SQLite connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self { config, status: ConnectionStatus::Disconnected }
    }

    /// Get database path
    fn get_db_path(&self) -> ConnectorResult<String> {
        self.config
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| ConnectorError::config("Database path not configured"))
    }
}

#[async_trait]
impl Connector for SqliteConnector {
    fn connector_type(&self) -> &str {
        "sqlite"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        // Verify configuration
        self.get_db_path()?;

        // In a real implementation, you would open the SQLite database here
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection(format!(
                "Database not connected: {}",
                self.status
            )));
        }

        let start = Instant::now();

        // Simulate query execution
        let result = match request.operation.to_uppercase().as_str() {
            "QUERY" => {
                let query = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("query"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Query not specified"))?;

                json!({
                    "query": query,
                    "rows_affected": 0,
                    "results": []
                })
            },
            "INSERT" | "UPDATE" | "DELETE" => {
                json!({
                    "operation": request.operation,
                    "rows_affected": 1
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
            return Ok(HealthStatus::Unhealthy(format!("Database status: {}", self.status)));
        }

        Ok(HealthStatus::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_postgres_connection_string() {
        let config = ConnectorConfig::new("test_pg", "postgresql")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(5432))
            .with_param("database", json!("testdb"))
            .with_param("user", json!("testuser"))
            .with_param("password", json!("testpass"));

        let connector = PostgresConnector::new(config);
        let conn_str = connector.get_connection_string().unwrap();
        assert!(conn_str.contains("postgresql://testuser:testpass@localhost:5432/testdb"));
    }

    #[test]
    fn test_mysql_connection_string() {
        let config = ConnectorConfig::new("test_mysql", "mysql")
            .with_param("host", json!("localhost"))
            .with_param("port", json!(3306))
            .with_param("database", json!("testdb"))
            .with_param("user", json!("testuser"))
            .with_param("password", json!("testpass"));

        let connector = MySqlConnector::new(config);
        let conn_str = connector.get_connection_string().unwrap();
        assert!(conn_str.contains("mysql://testuser:testpass@localhost:3306/testdb"));
    }

    #[test]
    fn test_sqlite_path() {
        let config =
            ConnectorConfig::new("test_sqlite", "sqlite").with_param("path", json!("/tmp/test.db"));

        let connector = SqliteConnector::new(config);
        let path = connector.get_db_path().unwrap();
        assert_eq!(path, "/tmp/test.db");
    }
}
