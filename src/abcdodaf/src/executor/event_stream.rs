//! Event Stream Module
//!
//! Real-time event streaming from executor to clients using PostgreSQL LISTEN/NOTIFY

use crate::executor::database::{ExecutionLog, DatabaseManager};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use sqlx::types::Uuid;
use tokio::sync::broadcast;

/// Event types that can be subscribed to
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Execution status changes
    ExecutionStatus,
    /// New log entries
    ExecutionLog,
    /// Connection health updates
    ConnectionHealth,
    /// Workflow updates
    WorkflowUpdate,
    /// DODAF compliance updates
    ComplianceUpdate,
}

/// Event that can be streamed to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Execution status changed
    ExecutionStatusChange {
        execution_id: Uuid,
        workflow_id: Uuid,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// New execution log
    ExecutionLog {
        execution_id: Uuid,
        level: String,
        source: Option<String>,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        context: Option<serde_json::Value>,
    },

    /// Connection health update
    ConnectionHealth {
        connection_type: String,
        connection_name: String,
        status: String,
        error_message: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Workflow updated
    WorkflowUpdate {
        workflow_id: Uuid,
        workflow_name: String,
        update_type: String,  // "created", "updated", "deleted"
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// DODAF compliance report updated
    ComplianceUpdate {
        workflow_id: Uuid,
        overall_score: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Event subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub event_types: Vec<EventType>,
    pub workflow_filter: Option<Uuid>,
    pub execution_filter: Option<Uuid>,
    pub log_level_filter: Option<String>,
}

impl Default for EventSubscription {
    fn default() -> Self {
        Self {
            event_types: vec![
                EventType::ExecutionStatus,
                EventType::ExecutionLog,
                EventType::ConnectionHealth,
            ],
            workflow_filter: None,
            execution_filter: None,
            log_level_filter: None,
        }
    }
}

/// Event stream manager
pub struct EventStream {
    db: DatabaseManager,
    broadcast_tx: broadcast::Sender<StreamEvent>,
}

impl EventStream {
    /// Create a new event stream
    pub fn new(db: DatabaseManager, buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self {
            db,
            broadcast_tx: tx,
        }
    }

    /// Start listening to PostgreSQL notifications
    pub async fn start_listening(&self) -> Result<()> {
        let mut listener = self.db.create_listener().await?;

        // Subscribe to notification channels
        listener.listen("execution_status_change").await?;
        listener.listen("execution_log").await?;
        listener.listen("connection_health_change").await?;

        // Clone sender for the task
        let tx = self.broadcast_tx.clone();

        // Spawn listener task
        tokio::spawn(async move {
            loop {
                match listener.recv().await {
                    Ok(notification) => {
                        let channel = notification.channel();
                        let payload = notification.payload();

                        // Parse notification and broadcast event
                        match channel {
                            "execution_status_change" => {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
                                    let event = StreamEvent::ExecutionStatusChange {
                                        execution_id: Uuid::parse_str(
                                            data["execution_id"].as_str().unwrap_or("")
                                        ).unwrap_or_default(),
                                        workflow_id: Uuid::parse_str(
                                            data["workflow_id"].as_str().unwrap_or("")
                                        ).unwrap_or_default(),
                                        status: data["status"].as_str().unwrap_or("").to_string(),
                                        timestamp: chrono::Utc::now(),
                                    };
                                    let _ = tx.send(event);
                                }
                            }
                            "execution_log" => {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
                                    let event = StreamEvent::ExecutionLog {
                                        execution_id: Uuid::parse_str(
                                            data["execution_id"].as_str().unwrap_or("")
                                        ).unwrap_or_default(),
                                        level: data["level"].as_str().unwrap_or("").to_string(),
                                        source: data["source"].as_str().map(|s| s.to_string()),
                                        message: data["message"].as_str().unwrap_or("").to_string(),
                                        timestamp: chrono::Utc::now(),
                                        context: data.get("context").cloned(),
                                    };
                                    let _ = tx.send(event);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving notification: {:?}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Subscribe to events with filter
    pub fn subscribe(&self, _filter: EventSubscription) -> broadcast::Receiver<StreamEvent> {
        // TODO: Implement filtering logic
        self.broadcast_tx.subscribe()
    }

    /// Manually broadcast an event
    pub fn broadcast(&self, event: StreamEvent) -> Result<()> {
        self.broadcast_tx.send(event)
            .map_err(|e| crate::error::AbcdodafError::Other(anyhow::anyhow!(e)))?;
        Ok(())
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.broadcast_tx.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = StreamEvent::ExecutionStatusChange {
            execution_id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            status: "running".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ExecutionStatusChange"));
    }

    #[test]
    fn test_event_subscription_default() {
        let sub = EventSubscription::default();
        assert_eq!(sub.event_types.len(), 3);
        assert!(sub.workflow_filter.is_none());
    }
}
