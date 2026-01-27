//! Health Monitoring Module
//!
//! Monitors health of all backend connections (PostgreSQL, PGVector, Qdrant, MCP)

use crate::error::Result;
use crate::executor::database::{ConnectionHealth, DatabaseManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Health monitor for backend connections
pub struct HealthMonitor {
    db: DatabaseManager,
    connections: Arc<RwLock<HashMap<String, ConnectionStatus>>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(db: DatabaseManager) -> Self {
        Self { db, connections: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Start health monitoring loop
    pub async fn start_monitoring(&self, interval: Duration) -> Result<()> {
        let db = self.db.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            loop {
                // Check PostgreSQL
                Self::check_postgres(&db, &connections).await;

                // Check PGVector
                Self::check_pgvector(&db, &connections).await;

                // Check Qdrant
                Self::check_qdrant(&db, &connections).await;

                // Check MCP servers (if configured)
                Self::check_mcp_servers(&db, &connections).await;

                // Check Ollama (if configured)
                Self::check_ollama(&db, &connections).await;

                tokio::time::sleep(interval).await;
            }
        });

        Ok(())
    }

    /// Check PostgreSQL connection
    async fn check_postgres(
        db: &DatabaseManager,
        connections: &Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    ) {
        let start = std::time::Instant::now();

        let status = match sqlx::query("SELECT 1").fetch_one(db.pool()).await {
            Ok(_) => ConnectionStatus {
                connection_type: "postgresql".to_string(),
                connection_name: "main".to_string(),
                status: "connected".to_string(),
                response_time_ms: start.elapsed().as_millis() as i32,
                error_message: None,
                last_check: chrono::Utc::now(),
            },
            Err(e) => ConnectionStatus {
                connection_type: "postgresql".to_string(),
                connection_name: "main".to_string(),
                status: "error".to_string(),
                response_time_ms: 0,
                error_message: Some(e.to_string()),
                last_check: chrono::Utc::now(),
            },
        };

        // Update database
        let health = ConnectionHealth {
            id: None,
            connection_type: status.connection_type.clone(),
            connection_name: status.connection_name.clone(),
            host: None,
            port: None,
            status: status.status.clone(),
            last_check_at: status.last_check,
            last_success_at: if status.status == "connected" {
                Some(status.last_check)
            } else {
                None
            },
            error_message: status.error_message.clone(),
            metadata: None,
        };

        let _ = db.update_connection_health(&health).await;

        // Update in-memory cache
        let mut conns = connections.write().await;
        conns.insert("postgresql:main".to_string(), status);
    }

    /// Check PGVector extension
    async fn check_pgvector(
        db: &DatabaseManager,
        connections: &Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    ) {
        let start = std::time::Instant::now();

        let status =
            match sqlx::query("SELECT extversion FROM pg_extension WHERE extname = 'vector'")
                .fetch_optional(db.pool())
                .await
            {
                Ok(Some(_)) => ConnectionStatus {
                    connection_type: "pgvector".to_string(),
                    connection_name: "extension".to_string(),
                    status: "connected".to_string(),
                    response_time_ms: start.elapsed().as_millis() as i32,
                    error_message: None,
                    last_check: chrono::Utc::now(),
                },
                Ok(None) => ConnectionStatus {
                    connection_type: "pgvector".to_string(),
                    connection_name: "extension".to_string(),
                    status: "disconnected".to_string(),
                    response_time_ms: 0,
                    error_message: Some("PGVector extension not installed".to_string()),
                    last_check: chrono::Utc::now(),
                },
                Err(e) => ConnectionStatus {
                    connection_type: "pgvector".to_string(),
                    connection_name: "extension".to_string(),
                    status: "error".to_string(),
                    response_time_ms: 0,
                    error_message: Some(e.to_string()),
                    last_check: chrono::Utc::now(),
                },
            };

        let health = ConnectionHealth {
            id: None,
            connection_type: status.connection_type.clone(),
            connection_name: status.connection_name.clone(),
            host: None,
            port: None,
            status: status.status.clone(),
            last_check_at: status.last_check,
            last_success_at: if status.status == "connected" {
                Some(status.last_check)
            } else {
                None
            },
            error_message: status.error_message.clone(),
            metadata: None,
        };

        let _ = db.update_connection_health(&health).await;

        let mut conns = connections.write().await;
        conns.insert("pgvector:extension".to_string(), status);
    }

    /// Check Qdrant connection
    async fn check_qdrant(
        db: &DatabaseManager,
        connections: &Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    ) {
        let start = std::time::Instant::now();

        // Get Qdrant URL from environment or config
        let qdrant_url = match std::env::var("QDRANT_URL") {
            Ok(url) => url,
            Err(_) => {
                // No Qdrant configured, skip check
                return;
            }
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();

        let status = match client {
            Ok(client) => {
                let health_url = format!("{}/healthz", qdrant_url.trim_end_matches('/'));
                match client.get(&health_url).send().await {
                    Ok(response) if response.status().is_success() => ConnectionStatus {
                        connection_type: "qdrant".to_string(),
                        connection_name: "main".to_string(),
                        status: "connected".to_string(),
                        response_time_ms: start.elapsed().as_millis() as i32,
                        error_message: None,
                        last_check: chrono::Utc::now(),
                    },
                    Ok(response) => ConnectionStatus {
                        connection_type: "qdrant".to_string(),
                        connection_name: "main".to_string(),
                        status: "error".to_string(),
                        response_time_ms: start.elapsed().as_millis() as i32,
                        error_message: Some(format!("HTTP {}", response.status())),
                        last_check: chrono::Utc::now(),
                    },
                    Err(e) => ConnectionStatus {
                        connection_type: "qdrant".to_string(),
                        connection_name: "main".to_string(),
                        status: "error".to_string(),
                        response_time_ms: 0,
                        error_message: Some(e.to_string()),
                        last_check: chrono::Utc::now(),
                    },
                }
            }
            Err(e) => ConnectionStatus {
                connection_type: "qdrant".to_string(),
                connection_name: "main".to_string(),
                status: "error".to_string(),
                response_time_ms: 0,
                error_message: Some(e.to_string()),
                last_check: chrono::Utc::now(),
            },
        };

        let health = ConnectionHealth {
            id: None,
            connection_type: status.connection_type.clone(),
            connection_name: status.connection_name.clone(),
            host: Some(qdrant_url),
            port: None,
            status: status.status.clone(),
            last_check_at: status.last_check,
            last_success_at: if status.status == "connected" {
                Some(status.last_check)
            } else {
                None
            },
            error_message: status.error_message.clone(),
            metadata: None,
        };

        let _ = db.update_connection_health(&health).await;

        let mut conns = connections.write().await;
        conns.insert("qdrant:main".to_string(), status);
    }

    /// Check MCP servers
    async fn check_mcp_servers(
        db: &DatabaseManager,
        connections: &Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    ) {
        // Get MCP server configurations from environment
        // Format: MCP_SERVERS=server1:command1:arg1,arg2;server2:command2:arg3
        let mcp_config = match std::env::var("MCP_SERVERS") {
            Ok(config) => config,
            Err(_) => {
                // No MCP servers configured, skip check
                return;
            }
        };

        for server_config in mcp_config.split(';') {
            if server_config.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = server_config.split(':').collect();
            if parts.is_empty() {
                continue;
            }

            let server_name = parts[0].trim();
            let start = std::time::Instant::now();

            // For MCP servers, we check if the process is responsive
            // In a full implementation, this would use the MCP protocol
            let status = ConnectionStatus {
                connection_type: "mcp".to_string(),
                connection_name: server_name.to_string(),
                status: "disconnected".to_string(),
                response_time_ms: start.elapsed().as_millis() as i32,
                error_message: Some("MCP health check not fully implemented".to_string()),
                last_check: chrono::Utc::now(),
            };

            let health = ConnectionHealth {
                id: None,
                connection_type: status.connection_type.clone(),
                connection_name: status.connection_name.clone(),
                host: None,
                port: None,
                status: status.status.clone(),
                last_check_at: status.last_check,
                last_success_at: None,
                error_message: status.error_message.clone(),
                metadata: None,
            };

            let _ = db.update_connection_health(&health).await;

            let mut conns = connections.write().await;
            conns.insert(format!("mcp:{}", server_name), status);
        }
    }

    /// Check Ollama connection
    async fn check_ollama(
        db: &DatabaseManager,
        connections: &Arc<RwLock<HashMap<String, ConnectionStatus>>>,
    ) {
        let start = std::time::Instant::now();

        // Get Ollama URL from environment or use default
        let ollama_url = std::env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();

        let status = match client {
            Ok(client) => {
                // Ollama health check endpoint
                let health_url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));
                match client.get(&health_url).send().await {
                    Ok(response) if response.status().is_success() => ConnectionStatus {
                        connection_type: "ollama".to_string(),
                        connection_name: "main".to_string(),
                        status: "connected".to_string(),
                        response_time_ms: start.elapsed().as_millis() as i32,
                        error_message: None,
                        last_check: chrono::Utc::now(),
                    },
                    Ok(response) => ConnectionStatus {
                        connection_type: "ollama".to_string(),
                        connection_name: "main".to_string(),
                        status: "error".to_string(),
                        response_time_ms: start.elapsed().as_millis() as i32,
                        error_message: Some(format!("HTTP {}", response.status())),
                        last_check: chrono::Utc::now(),
                    },
                    Err(e) => ConnectionStatus {
                        connection_type: "ollama".to_string(),
                        connection_name: "main".to_string(),
                        status: "disconnected".to_string(),
                        response_time_ms: 0,
                        error_message: Some(e.to_string()),
                        last_check: chrono::Utc::now(),
                    },
                }
            }
            Err(e) => ConnectionStatus {
                connection_type: "ollama".to_string(),
                connection_name: "main".to_string(),
                status: "error".to_string(),
                response_time_ms: 0,
                error_message: Some(e.to_string()),
                last_check: chrono::Utc::now(),
            },
        };

        let health = ConnectionHealth {
            id: None,
            connection_type: status.connection_type.clone(),
            connection_name: status.connection_name.clone(),
            host: Some(ollama_url),
            port: None,
            status: status.status.clone(),
            last_check_at: status.last_check,
            last_success_at: if status.status == "connected" {
                Some(status.last_check)
            } else {
                None
            },
            error_message: status.error_message.clone(),
            metadata: None,
        };

        let _ = db.update_connection_health(&health).await;

        let mut conns = connections.write().await;
        conns.insert("ollama:main".to_string(), status);
    }

    /// Get current connection statuses
    pub async fn get_statuses(&self) -> HashMap<String, ConnectionStatus> {
        self.connections.read().await.clone()
    }

    /// Get overall health status
    pub async fn overall_health(&self) -> OverallHealth {
        let statuses = self.get_statuses().await;

        let total = statuses.len();
        let connected = statuses.values().filter(|s| s.status == "connected").count();
        let error = statuses.values().filter(|s| s.status == "error").count();
        let disconnected = statuses.values().filter(|s| s.status == "disconnected").count();

        let health_percentage =
            if total > 0 { (connected as f64 / total as f64) * 100.0 } else { 0.0 };

        OverallHealth {
            total_connections: total,
            connected,
            disconnected,
            error,
            health_percentage,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Connection status snapshot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionStatus {
    pub connection_type: String,
    pub connection_name: String,
    pub status: String,
    pub response_time_ms: i32,
    pub error_message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Overall health summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverallHealth {
    pub total_connections: usize,
    pub connected: usize,
    pub disconnected: usize,
    pub error: usize,
    pub health_percentage: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_status_serialization() {
        let status = ConnectionStatus {
            connection_type: "postgresql".to_string(),
            connection_name: "main".to_string(),
            status: "connected".to_string(),
            response_time_ms: 5,
            error_message: None,
            last_check: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("postgresql"));
    }
}
