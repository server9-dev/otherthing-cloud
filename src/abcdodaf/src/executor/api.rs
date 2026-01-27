//! API Server Module
//!
//! HTTP REST API for executor daemon

use crate::executor::database::{ExecutionRecord, StoredWorkflow};
use crate::executor::{DatabaseManager, DodafTracker, EventStream, HealthMonitor};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::sync::Arc;

/// API server state
#[derive(Clone)]
pub struct ApiState {
    db: DatabaseManager,
    event_stream: Arc<EventStream>,
    health_monitor: Arc<HealthMonitor>,
    dodaf_tracker: Arc<DodafTracker>,
}

impl ApiState {
    pub fn new(
        db: DatabaseManager,
        event_stream: Arc<EventStream>,
        health_monitor: Arc<HealthMonitor>,
        dodaf_tracker: Arc<DodafTracker>,
    ) -> Self {
        Self { db, event_stream, health_monitor, dodaf_tracker }
    }
}

/// API server
pub struct ApiServer {
    state: ApiState,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(
        db: DatabaseManager,
        event_stream: Arc<EventStream>,
        health_monitor: Arc<HealthMonitor>,
        dodaf_tracker: Arc<DodafTracker>,
    ) -> Self {
        let state = ApiState::new(db, event_stream, health_monitor, dodaf_tracker);
        Self { state }
    }

    /// Create router
    pub fn router(&self) -> Router {
        Router::new()
            // Workflow management
            .route("/api/workflows", post(submit_workflow))
            .route("/api/workflows", get(list_workflows))
            .route("/api/workflows/:id", get(get_workflow))

            // Health and status
            .route("/api/health", get(get_health))
            .route("/api/health/connections", get(get_connection_health))

            .with_state(self.state.clone())
    }

    /// Start API server
    pub async fn serve(&self, addr: &str) -> crate::error::Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

// Route handlers

/// Submit a workflow for execution
async fn submit_workflow(
    State(state): State<ApiState>,
    Json(payload): Json<SubmitWorkflowRequest>,
) -> Response {
    // Store workflow
    let workflow = StoredWorkflow {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        version: payload.version.unwrap_or(1),
        description: payload.description.clone(),
        workflow_json: payload.workflow_json,
        workflow_type: payload.workflow_type,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: payload.created_by.clone(),
        is_active: true,
        tags: payload.tags.unwrap_or_default(),
        metadata: payload.metadata,
        last_validation_at: None,
        validation_passed: None,
        validation_report: None,
        dodaf_compliance_score: None,
        dodaf_compliance_level: None,
    };

    match state.db.store_workflow(&workflow).await {
        Ok(workflow_id) => {
            // Create execution if auto_execute is true
            let execution_id = if payload.auto_execute.unwrap_or(false) {
                let execution = ExecutionRecord {
                    workflow_id,
                    workflow_version: workflow.version,
                    status: "pending".to_string(),
                    triggered_by: payload.created_by,
                    trigger_type: Some("api".to_string()),
                    input_data: payload.input_data,
                    tasks_total: None,
                    bpmn_instance_id: None,
                };

                match state.db.create_execution(&execution).await {
                    Ok(id) => Some(id),
                    Err(_) => None,
                }
            } else {
                None
            };

            Json(SubmitWorkflowResponse { workflow_id, execution_id }).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

/// List workflows
async fn list_workflows(
    State(state): State<ApiState>,
    Query(params): Query<ListWorkflowsQuery>,
) -> Response {
    match state.db.list_workflows(params.active_only.unwrap_or(true)).await {
        Ok(workflows) => Json(workflows).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

/// Get workflow by ID
async fn get_workflow(State(state): State<ApiState>, Path(id): Path<Uuid>) -> Response {
    match state.db.get_workflow(id).await {
        Ok(Some(workflow)) => Json(workflow).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Workflow not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

/// Get health status
async fn get_health(State(state): State<ApiState>) -> Response {
    let health = state.health_monitor.overall_health().await;
    Json(health).into_response()
}

/// Get connection health
async fn get_connection_health(State(state): State<ApiState>) -> Response {
    match state.db.get_connection_health().await {
        Ok(connections) => Json(connections).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
struct SubmitWorkflowRequest {
    name: String,
    version: Option<i32>,
    description: Option<String>,
    workflow_json: serde_json::Value,
    workflow_type: String,
    created_by: Option<String>,
    tags: Option<Vec<String>>,
    metadata: Option<serde_json::Value>,
    auto_execute: Option<bool>,
    input_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct SubmitWorkflowResponse {
    workflow_id: Uuid,
    execution_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct ListWorkflowsQuery {
    active_only: Option<bool>,
}
