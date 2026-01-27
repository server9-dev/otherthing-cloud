//! Executor Daemon Module
//!
//! Persistent service that:
//! - Manages workflow storage in PostgreSQL
//! - Executes workflows from database
//! - Connects to all backends (PGVector, Qdrant, MCP services)
//! - Tracks DODAF metadata and compliance
//! - Streams logs and status to connected clients

pub mod api;
pub mod daemon;
pub mod database;
pub mod dodaf_tracker;
pub mod event_stream;
pub mod health;

pub use api::ApiServer;
pub use daemon::{ExecutorConfig, ExecutorDaemon};
pub use database::{ConnectionHealth, DatabaseManager};
pub use dodaf_tracker::DodafTracker;
pub use event_stream::{EventStream, EventSubscription, EventType};
pub use health::HealthMonitor;

use crate::error::AbcdodafError;

pub type ExecutorResult<T> = Result<T, AbcdodafError>;
