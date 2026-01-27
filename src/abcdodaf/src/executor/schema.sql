-- ABCDODAF Executor Database Schema
-- PostgreSQL schema for workflow storage and execution tracking

-- Workflows table (source of truth)
CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    workflow_json JSONB NOT NULL,  -- Full BPMN/Snarl JSON
    workflow_type VARCHAR(50) NOT NULL,  -- 'bpmn', 'snarl', 'bpmn_xml'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tags TEXT[],
    metadata JSONB,

    -- Validation results
    last_validation_at TIMESTAMPTZ,
    validation_passed BOOLEAN,
    validation_report JSONB,

    -- DODAF compliance
    dodaf_compliance_score FLOAT,
    dodaf_compliance_level VARCHAR(50),

    CONSTRAINT unique_workflow_version UNIQUE (name, version)
);

CREATE INDEX idx_workflows_name ON workflows(name);
CREATE INDEX idx_workflows_active ON workflows(is_active);
CREATE INDEX idx_workflows_type ON workflows(workflow_type);
CREATE INDEX idx_workflows_tags ON workflows USING GIN(tags);
CREATE INDEX idx_workflows_created ON workflows(created_at DESC);

-- Workflow executions
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    workflow_version INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL,  -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,

    -- Execution context
    triggered_by VARCHAR(255),
    trigger_type VARCHAR(50),  -- 'manual', 'scheduled', 'event', 'api'
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,

    -- Performance metrics
    tasks_total INTEGER,
    tasks_completed INTEGER,
    tasks_failed INTEGER,

    -- Process instance tracking
    bpmn_instance_id VARCHAR(255),

    CONSTRAINT valid_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled'))
);

CREATE INDEX idx_executions_workflow ON workflow_executions(workflow_id);
CREATE INDEX idx_executions_status ON workflow_executions(status);
CREATE INDEX idx_executions_started ON workflow_executions(started_at DESC);
CREATE INDEX idx_executions_bpmn_instance ON workflow_executions(bpmn_instance_id);

-- Execution logs (real-time streaming)
CREATE TABLE IF NOT EXISTS execution_logs (
    id BIGSERIAL PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level VARCHAR(20) NOT NULL,  -- 'trace', 'debug', 'info', 'warn', 'error', 'fatal'
    source VARCHAR(255),  -- Task ID, process ID, etc.
    message TEXT NOT NULL,
    context JSONB,

    CONSTRAINT valid_log_level CHECK (level IN ('trace', 'debug', 'info', 'warn', 'error', 'fatal'))
);

CREATE INDEX idx_logs_execution ON execution_logs(execution_id, timestamp DESC);
CREATE INDEX idx_logs_level ON execution_logs(level);
CREATE INDEX idx_logs_timestamp ON execution_logs(timestamp DESC);

-- Task executions (individual BPMN tasks)
CREATE TABLE IF NOT EXISTS task_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    task_id VARCHAR(255) NOT NULL,
    task_name VARCHAR(255),
    task_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,

    -- Task data
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,

    -- Agent/performer tracking
    assigned_to VARCHAR(255),
    performer_type VARCHAR(50),  -- 'agent', 'human', 'system'

    CONSTRAINT valid_task_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'skipped'))
);

CREATE INDEX idx_task_executions_execution ON task_executions(execution_id);
CREATE INDEX idx_task_executions_task ON task_executions(task_id);
CREATE INDEX idx_task_executions_status ON task_executions(status);

-- System connections (health monitoring)
CREATE TABLE IF NOT EXISTS system_connections (
    id SERIAL PRIMARY KEY,
    connection_type VARCHAR(100) NOT NULL,  -- 'postgresql', 'pgvector', 'qdrant', 'mcp_server', 'ollama'
    connection_name VARCHAR(255) NOT NULL,
    host VARCHAR(255),
    port INTEGER,
    status VARCHAR(50) NOT NULL,  -- 'connected', 'disconnected', 'error', 'degraded'
    last_check_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_success_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB,

    UNIQUE(connection_type, connection_name)
);

CREATE INDEX idx_connections_status ON system_connections(status);
CREATE INDEX idx_connections_type ON system_connections(connection_type);

-- Connection health history
CREATE TABLE IF NOT EXISTS connection_health_history (
    id BIGSERIAL PRIMARY KEY,
    connection_id INTEGER NOT NULL REFERENCES system_connections(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL,
    response_time_ms INTEGER,
    error_message TEXT
);

CREATE INDEX idx_health_history_connection ON connection_health_history(connection_id, timestamp DESC);

-- Validation results cache
CREATE TABLE IF NOT EXISTS validation_cache (
    workflow_id UUID PRIMARY KEY REFERENCES workflows(id) ON DELETE CASCADE,
    validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    validation_result JSONB NOT NULL,
    compliance_report JSONB
);

-- DODAF metadata storage
CREATE TABLE IF NOT EXISTS dodaf_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    view_type VARCHAR(50) NOT NULL,  -- 'OV-1', 'OV-2', 'OV-3', 'OV-5', etc.
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(workflow_id, view_type)
);

CREATE INDEX idx_dodaf_workflow ON dodaf_metadata(workflow_id);
CREATE INDEX idx_dodaf_view ON dodaf_metadata(view_type);

-- Event subscriptions (for client connections)
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) NOT NULL,
    subscription_type VARCHAR(100) NOT NULL,  -- 'execution_logs', 'status_updates', 'connection_health'
    filter_json JSONB,  -- Optional filters (e.g., specific workflow_id, log level)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_subscriptions_client ON event_subscriptions(client_id);
CREATE INDEX idx_subscriptions_type ON event_subscriptions(subscription_type);
CREATE INDEX idx_subscriptions_active ON event_subscriptions(is_active);

-- Scheduled workflows
CREATE TABLE IF NOT EXISTS workflow_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    schedule_name VARCHAR(255) NOT NULL,
    cron_expression VARCHAR(255) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    timezone VARCHAR(100) DEFAULT 'UTC',

    -- Execution tracking
    last_execution_at TIMESTAMPTZ,
    next_execution_at TIMESTAMPTZ,
    execution_count BIGINT NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_schedules_workflow ON workflow_schedules(workflow_id);
CREATE INDEX idx_schedules_next_execution ON workflow_schedules(next_execution_at) WHERE is_enabled = TRUE;

-- PGVector: Workflow embeddings for semantic search
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS workflow_embeddings (
    workflow_id UUID PRIMARY KEY REFERENCES workflows(id) ON DELETE CASCADE,
    embedding vector(1536),  -- Assuming OpenAI ada-002 dimensions
    embedding_model VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_embeddings_vector ON workflow_embeddings
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_workflows_updated_at BEFORE UPDATE ON workflows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dodaf_metadata_updated_at BEFORE UPDATE ON dodaf_metadata
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_workflow_schedules_updated_at BEFORE UPDATE ON workflow_schedules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE OR REPLACE VIEW active_executions AS
SELECT
    e.id,
    e.workflow_id,
    w.name AS workflow_name,
    e.status,
    e.started_at,
    EXTRACT(EPOCH FROM (NOW() - e.started_at))::BIGINT AS running_seconds,
    e.tasks_completed,
    e.tasks_total,
    CASE
        WHEN e.tasks_total > 0 THEN (e.tasks_completed::FLOAT / e.tasks_total * 100)
        ELSE 0
    END AS progress_percent
FROM workflow_executions e
JOIN workflows w ON e.workflow_id = w.id
WHERE e.status IN ('pending', 'running');

CREATE OR REPLACE VIEW connection_status_summary AS
SELECT
    connection_type,
    COUNT(*) AS total_connections,
    COUNT(*) FILTER (WHERE status = 'connected') AS connected,
    COUNT(*) FILTER (WHERE status = 'disconnected') AS disconnected,
    COUNT(*) FILTER (WHERE status = 'error') AS error,
    MAX(last_check_at) AS last_check
FROM system_connections
GROUP BY connection_type;

-- Notification triggers for real-time updates (using PostgreSQL LISTEN/NOTIFY)
CREATE OR REPLACE FUNCTION notify_execution_status_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'execution_status_change',
        json_build_object(
            'execution_id', NEW.id,
            'workflow_id', NEW.workflow_id,
            'status', NEW.status,
            'timestamp', NOW()
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER execution_status_notification
AFTER INSERT OR UPDATE OF status ON workflow_executions
FOR EACH ROW EXECUTE FUNCTION notify_execution_status_change();

CREATE OR REPLACE FUNCTION notify_new_log()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'execution_log',
        json_build_object(
            'execution_id', NEW.execution_id,
            'level', NEW.level,
            'message', NEW.message,
            'timestamp', NEW.timestamp
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER execution_log_notification
AFTER INSERT ON execution_logs
FOR EACH ROW EXECUTE FUNCTION notify_new_log();
