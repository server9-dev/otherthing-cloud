//! Database Manager for Executor
//!
//! Handles all PostgreSQL interactions including:
//! - Workflow CRUD operations
//! - Execution tracking
//! - Log storage
//! - DODAF metadata persistence

use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgListener, PgPool, PgPoolOptions};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use sqlx::{FromRow, Row};
use std::time::Duration;

/// Database manager for executor
#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await
            .map_err(|e| AbcdodafError::Other(e.into()))?;

        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn initialize_schema(&self) -> Result<()> {
        let schema = include_str!("schema.sql");
        sqlx::query(schema)
            .execute(&self.pool)
            .await
            .map_err(|e| AbcdodafError::Other(e.into()))?;
        Ok(())
    }

    /// Create a PostgreSQL listener for notifications
    pub async fn create_listener(&self) -> Result<PgListener> {
        PgListener::connect_with(&self.pool)
            .await
            .map_err(|e| AbcdodafError::Other(e.into()))
    }

    /// Store a workflow
    pub async fn store_workflow(&self, workflow: &StoredWorkflow) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO workflows (
                name, version, description, workflow_json, workflow_type,
                created_by, tags, metadata, validation_passed, validation_report,
                dodaf_compliance_score, dodaf_compliance_level
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
        )
        .bind(&workflow.name)
        .bind(workflow.version)
        .bind(&workflow.description)
        .bind(&workflow.workflow_json)
        .bind(&workflow.workflow_type)
        .bind(&workflow.created_by)
        .bind(&workflow.tags)
        .bind(&workflow.metadata)
        .bind(workflow.validation_passed)
        .bind(&workflow.validation_report)
        .bind(workflow.dodaf_compliance_score)
        .bind(&workflow.dodaf_compliance_level)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Get a workflow by ID
    pub async fn get_workflow(&self, id: Uuid) -> Result<Option<StoredWorkflow>> {
        let workflow = sqlx::query_as::<_, StoredWorkflow>(
            r#"
            SELECT
                id, name, version, description, workflow_json, workflow_type,
                created_at, updated_at, created_by, is_active, tags, metadata,
                last_validation_at, validation_passed, validation_report,
                dodaf_compliance_score, dodaf_compliance_level
            FROM workflows
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workflow)
    }

    /// Get workflow by name and version
    pub async fn get_workflow_by_name(
        &self,
        name: &str,
        version: i32,
    ) -> Result<Option<StoredWorkflow>> {
        let workflow = sqlx::query_as::<_, StoredWorkflow>(
            r#"
            SELECT
                id, name, version, description, workflow_json, workflow_type,
                created_at, updated_at, created_by, is_active, tags, metadata,
                last_validation_at, validation_passed, validation_report,
                dodaf_compliance_score, dodaf_compliance_level
            FROM workflows
            WHERE name = $1 AND version = $2
            "#,
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workflow)
    }

    /// List all workflows
    pub async fn list_workflows(&self, active_only: bool) -> Result<Vec<WorkflowSummary>> {
        let query = if active_only {
            r#"
            SELECT id, name, version, description, workflow_type, created_at, updated_at,
                   dodaf_compliance_score, dodaf_compliance_level
            FROM workflows
            WHERE is_active = true
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT id, name, version, description, workflow_type, created_at, updated_at,
                   dodaf_compliance_score, dodaf_compliance_level
            FROM workflows
            ORDER BY created_at DESC
            "#
        };

        let workflows = sqlx::query_as::<_, WorkflowSummary>(query).fetch_all(&self.pool).await?;

        Ok(workflows)
    }

    /// Create a workflow execution record
    pub async fn create_execution(&self, execution: &ExecutionRecord) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO workflow_executions (
                workflow_id, workflow_version, status, triggered_by, trigger_type,
                input_data, tasks_total, bpmn_instance_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
        .bind(execution.workflow_id)
        .bind(execution.workflow_version)
        .bind(&execution.status)
        .bind(&execution.triggered_by)
        .bind(&execution.trigger_type)
        .bind(&execution.input_data)
        .bind(execution.tasks_total)
        .bind(&execution.bpmn_instance_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Update execution status
    pub async fn update_execution_status(
        &self,
        execution_id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE workflow_executions
            SET status = $2,
                completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE NULL END,
                duration_ms = CASE WHEN $2 IN ('completed', 'failed', 'cancelled')
                                   THEN EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000
                                   ELSE NULL END,
                error_message = $3
            WHERE id = $1
            "#
        )
        .bind(execution_id)
        .bind(status)
        .bind(error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Add execution log
    pub async fn add_execution_log(&self, log: &ExecutionLog) -> Result<i64> {
        let row = sqlx::query(
            r#"
            INSERT INTO execution_logs (execution_id, level, source, message, context)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(log.execution_id)
        .bind(&log.level)
        .bind(&log.source)
        .bind(&log.message)
        .bind(&log.context)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Get execution logs
    pub async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        level_filter: Option<String>,
        limit: i64,
    ) -> Result<Vec<ExecutionLog>> {
        let logs = if let Some(level) = level_filter {
            sqlx::query_as::<_, ExecutionLog>(
                r#"
                SELECT id, execution_id, timestamp, level, source, message, context
                FROM execution_logs
                WHERE execution_id = $1 AND level = $2
                ORDER BY timestamp DESC
                LIMIT $3
                "#,
            )
            .bind(execution_id)
            .bind(level)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ExecutionLog>(
                r#"
                SELECT id, execution_id, timestamp, level, source, message, context
                FROM execution_logs
                WHERE execution_id = $1
                ORDER BY timestamp DESC
                LIMIT $2
                "#,
            )
            .bind(execution_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(logs)
    }

    /// Store DODAF metadata
    pub async fn store_dodaf_metadata(
        &self,
        workflow_id: Uuid,
        view_type: &str,
        metadata: serde_json::Value,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO dodaf_metadata (workflow_id, view_type, metadata)
            VALUES ($1, $2, $3)
            ON CONFLICT (workflow_id, view_type)
            DO UPDATE SET metadata = $3, updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(workflow_id)
        .bind(view_type)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Get DODAF metadata for a workflow
    pub async fn get_dodaf_metadata(
        &self,
        workflow_id: Uuid,
        view_type: Option<&str>,
    ) -> Result<Vec<DodafMetadataRecord>> {
        let records = if let Some(vt) = view_type {
            sqlx::query_as::<_, DodafMetadataRecord>(
                r#"
                SELECT id, workflow_id, view_type, metadata, created_at, updated_at
                FROM dodaf_metadata
                WHERE workflow_id = $1 AND view_type = $2
                "#,
            )
            .bind(workflow_id)
            .bind(vt)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, DodafMetadataRecord>(
                r#"
                SELECT id, workflow_id, view_type, metadata, created_at, updated_at
                FROM dodaf_metadata
                WHERE workflow_id = $1
                "#,
            )
            .bind(workflow_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(records)
    }

    /// Update connection health
    pub async fn update_connection_health(&self, health: &ConnectionHealth) -> Result<i32> {
        let row = sqlx::query(
            r#"
            INSERT INTO system_connections (connection_type, connection_name, host, port, status, error_message, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (connection_type, connection_name)
            DO UPDATE SET
                status = $5,
                last_check_at = NOW(),
                last_success_at = CASE WHEN $5 = 'connected' THEN NOW() ELSE system_connections.last_success_at END,
                error_message = $6,
                metadata = $7
            RETURNING id
            "#
        )
        .bind(&health.connection_type)
        .bind(&health.connection_name)
        .bind(&health.host)
        .bind(health.port)
        .bind(&health.status)
        .bind(&health.error_message)
        .bind(&health.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Get all connection health statuses
    pub async fn get_connection_health(&self) -> Result<Vec<ConnectionHealth>> {
        let connections = sqlx::query_as::<_, ConnectionHealth>(
            r#"
            SELECT
                id, connection_type, connection_name, host, port, status,
                last_check_at, last_success_at, error_message, metadata
            FROM system_connections
            ORDER BY connection_type, connection_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Get pool reference for advanced queries
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Count active executions
    pub async fn count_active_executions(&self) -> Result<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM workflow_executions
            WHERE status IN ('running', 'pending')
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AbcdodafError::Other(e.into()))?;

        let count: i64 = row.try_get("count")
            .map_err(|e| AbcdodafError::Other(e.into()))?;

        Ok(count)
    }

    /// Close database connections
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

/// Stored workflow record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredWorkflow {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub workflow_json: serde_json::Value,
    pub workflow_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub last_validation_at: Option<DateTime<Utc>>,
    pub validation_passed: Option<bool>,
    pub validation_report: Option<serde_json::Value>,
    pub dodaf_compliance_score: Option<f64>,
    pub dodaf_compliance_level: Option<String>,
}

/// Workflow summary (for listing)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowSummary {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub description: Option<String>,
    pub workflow_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dodaf_compliance_score: Option<f64>,
    pub dodaf_compliance_level: Option<String>,
}

/// Execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub workflow_id: Uuid,
    pub workflow_version: i32,
    pub status: String,
    pub triggered_by: Option<String>,
    pub trigger_type: Option<String>,
    pub input_data: Option<serde_json::Value>,
    pub tasks_total: Option<i32>,
    pub bpmn_instance_id: Option<String>,
}

/// Execution log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub execution_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub source: Option<String>,
    pub message: String,
    pub context: Option<serde_json::Value>,
}

/// DODAF metadata record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DodafMetadataRecord {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub view_type: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Connection health status
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConnectionHealth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub connection_type: String,
    pub connection_name: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub status: String,
    pub last_check_at: DateTime<Utc>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL
    async fn test_database_manager() {
        let db = DatabaseManager::new("postgresql://localhost/abcdodaf_test").await.unwrap();

        db.initialize_schema().await.unwrap();

        // Test workflow storage
        let workflow = StoredWorkflow {
            id: Uuid::new_v4(),
            name: "test_workflow".to_string(),
            version: 1,
            description: Some("Test workflow".to_string()),
            workflow_json: serde_json::json!({"test": "data"}),
            workflow_type: "bpmn".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            is_active: true,
            tags: vec!["test".to_string()],
            metadata: None,
            last_validation_at: None,
            validation_passed: None,
            validation_report: None,
            dodaf_compliance_score: None,
            dodaf_compliance_level: None,
        };

        let id = db.store_workflow(&workflow).await.unwrap();
        assert_ne!(id, Uuid::nil());
    }
}
