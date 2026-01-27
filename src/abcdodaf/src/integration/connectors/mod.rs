//! Concrete connector implementations

pub mod database;
pub mod filesystem;
pub mod pgvector;
pub mod qdrant;
pub mod rest_api;
pub mod webhook;

pub use database::{MySqlConnector, PostgresConnector, SqliteConnector};
pub use filesystem::FileSystemConnector;
pub use pgvector::PgVectorConnector;
pub use qdrant::QdrantConnector;
pub use rest_api::RestApiConnector;
pub use webhook::{IncomingWebhookConnector, OutgoingWebhookConnector, WebhookEvent};
