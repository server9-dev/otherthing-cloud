//! Concrete connector implementations

pub mod rest_api;
pub mod database;
pub mod webhook;
pub mod filesystem;
pub mod pgvector;
pub mod qdrant;

pub use rest_api::RestApiConnector;
pub use database::{PostgresConnector, MySqlConnector, SqliteConnector};
pub use webhook::{OutgoingWebhookConnector, IncomingWebhookConnector, WebhookEvent};
pub use filesystem::FileSystemConnector;
pub use pgvector::PgVectorConnector;
pub use qdrant::QdrantConnector;
