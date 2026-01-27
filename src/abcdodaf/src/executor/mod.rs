//! Executor Daemon Module
//!
//! Persistent service that:
//! - Manages workflow storage in PostgreSQL
//! - Executes workflows from database
//! - Connects to all backends (PGVector, Qdrant, MCP services)
//! - Tracks DODAF metadata and compliance
//! - Streams logs and status to connected clients

pub mod daemon;
pub mod database;
pub mod api;
pub mod event_stream;
pub mod health;
pub mod dodaf_tracker;

pub use daemon::{ExecutorDaemon, ExecutorConfig};
pub use database::{DatabaseManager, ConnectionHealth};
pub use api::ApiServer;
pub use event_stream::{EventStream, EventSubscription, EventType};
pub use health::HealthMonitor;
pub use dodaf_tracker::DodafTracker;

use crate::error::AbcdodafError;

pub type ExecutorResult<T> = Result<T, AbcdodafError>;
