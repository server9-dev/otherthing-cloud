//! Concrete connector implementations

pub mod rest_api;
pub mod database;
pub mod webhook;
pub mod filesystem;

pub use rest_api::RestApiConnector;
pub use database::{PostgresConnector, MySqlConnector, SqliteConnector};
pub use webhook::{OutgoingWebhookConnector, IncomingWebhookConnector, WebhookEvent};
pub use filesystem::FileSystemConnector;
