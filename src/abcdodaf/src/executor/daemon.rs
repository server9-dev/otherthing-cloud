//! Executor Daemon
//!
//! Main daemon process that coordinates all executor components

use crate::executor::{
    ApiServer, DatabaseManager, EventStream, HealthMonitor, DodafTracker,
};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Executor daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// PostgreSQL connection URL
    pub database_url: String,

    /// API server bind address
    pub api_bind: String,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Event stream buffer size
    pub event_buffer_size: usize,

    /// Qdrant configuration (optional)
    pub qdrant_url: Option<String>,

    /// Ollama configuration (optional)
    pub ollama_url: Option<String>,

    /// MCP server configurations
    pub mcp_servers: Vec<McpServerConfig>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/abcdodaf".to_string()),
            api_bind: "127.0.0.1:8080".to_string(),
            health_check_interval_secs: 30,
            event_buffer_size: 1000,
            qdrant_url: std::env::var("QDRANT_URL").ok(),
            ollama_url: std::env::var("OLLAMA_URL").ok(),
            mcp_servers: Vec::new(),
        }
    }
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

/// Executor daemon
pub struct ExecutorDaemon {
    config: ExecutorConfig,
    db: DatabaseManager,
    event_stream: Arc<EventStream>,
    health_monitor: Arc<HealthMonitor>,
    dodaf_tracker: Arc<DodafTracker>,
    api_server: ApiServer,
}

impl ExecutorDaemon {
    /// Create a new executor daemon
    pub async fn new(config: ExecutorConfig) -> Result<Self> {
        // Initialize database
        let db = DatabaseManager::new(&config.database_url).await?;
        db.initialize_schema().await?;

        // Create event stream
        let event_stream = Arc::new(EventStream::new(db.clone(), config.event_buffer_size));

        // Create health monitor
        let health_monitor = Arc::new(HealthMonitor::new(db.clone()));

        // Create DODAF tracker
        let dodaf_tracker = Arc::new(DodafTracker::new(db.clone()));

        // Create API server
        let api_server = ApiServer::new(
            db.clone(),
            event_stream.clone(),
            health_monitor.clone(),
            dodaf_tracker.clone(),
        );

        Ok(Self {
            config,
            db,
            event_stream,
            health_monitor,
            dodaf_tracker,
            api_server,
        })
    }

    /// Start the executor daemon
    pub async fn start(&self) -> Result<()> {
        println!("🚀 Starting ABCDODAF Executor Daemon");
        println!("   Database: {}", self.config.database_url);
        println!("   API: http://{}", self.config.api_bind);
        println!();

        // Start event stream listener
        println!("📡 Starting event stream...");
        self.event_stream.start_listening().await?;

        // Start health monitor
        println!("🏥 Starting health monitor...");
        self.health_monitor
            .start_monitoring(Duration::from_secs(self.config.health_check_interval_secs))
            .await?;

        // Start API server
        println!("🌐 Starting API server on {}...", self.config.api_bind);
        println!();
        println!("✅ Executor daemon is running");
        println!();
        println!("API Endpoints:");
        println!("  - POST   /api/workflows          Submit workflow");
        println!("  - GET    /api/workflows          List workflows");
        println!("  - GET    /api/workflows/:id      Get workflow");
        println!("  - GET    /api/executions/:id     Get execution status");
        println!("  - GET    /api/executions/:id/logs Get execution logs");
        println!("  - GET    /api/executions/:id/stream Stream logs (SSE)");
        println!("  - GET    /api/health             Overall health");
        println!("  - GET    /api/health/connections Connection health");
        println!("  - GET    /api/workflows/:id/dodaf DODAF metadata");
        println!("  - GET    /api/workflows/:id/compliance Compliance report");
        println!("  - GET    /api/events/stream      Event stream (SSE)");
        println!();

        self.api_server.serve(&self.config.api_bind).await?;

        Ok(())
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        println!("🛑 Shutting down executor daemon...");
        // TODO: Implement graceful shutdown
        // - Stop accepting new workflows
        // - Wait for current executions to complete
        // - Close database connections
        // - Stop health monitoring
        Ok(())
    }

    /// Get database manager reference
    pub fn database(&self) -> &DatabaseManager {
        &self.db
    }

    /// Get event stream reference
    pub fn event_stream(&self) -> &Arc<EventStream> {
        &self.event_stream
    }

    /// Get health monitor reference
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }

    /// Get DODAF tracker reference
    pub fn dodaf_tracker(&self) -> &Arc<DodafTracker> {
        &self.dodaf_tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExecutorConfig::default();
        assert_eq!(config.api_bind, "127.0.0.1:8080");
        assert_eq!(config.health_check_interval_secs, 30);
        assert_eq!(config.event_buffer_size, 1000);
    }

    #[test]
    fn test_config_serialization() {
        let config = ExecutorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let _deserialized: ExecutorConfig = serde_json::from_str(&json).unwrap();
    }
}
