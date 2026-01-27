//! File system connector for file operations

use crate::integration::connector::{
    Connector, ConnectionStatus, ConnectorConfig, ConnectorError, ConnectorRequest,
    ConnectorResponse, ConnectorResult, HealthStatus,
};
use async_trait::async_trait;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;

/// File system operations connector
pub struct FileSystemConnector {
    config: ConnectorConfig,
    status: ConnectionStatus,
    base_path: Option<PathBuf>,
}

impl FileSystemConnector {
    /// Create a new file system connector
    pub fn new(config: ConnectorConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            base_path: None,
        }
    }

    /// Get the base path for file operations
    fn get_base_path(&self) -> ConnectorResult<PathBuf> {
        self.config
            .params
            .get("base_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .ok_or_else(|| ConnectorError::config("Base path not configured"))
    }

    /// Ensure path is within base path (security check)
    fn validate_path(&self, path: &Path) -> ConnectorResult<PathBuf> {
        let base = self.base_path.as_ref().ok_or_else(|| {
            ConnectorError::connection("Base path not initialized")
        })?;

        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        };

        // Security check: ensure the path is within base_path
        if !full_path.starts_with(base) {
            return Err(ConnectorError::validation(
                "Path escapes base directory".to_string(),
            ));
        }

        Ok(full_path)
    }

    /// Check if read operations are allowed
    fn allow_read(&self) -> bool {
        self.config
            .config
            .get("allow_read")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// Check if write operations are allowed
    fn allow_write(&self) -> bool {
        self.config
            .config
            .get("allow_write")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// Check if delete operations are allowed
    fn allow_delete(&self) -> bool {
        self.config
            .config
            .get("allow_delete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[async_trait]
impl Connector for FileSystemConnector {
    fn connector_type(&self) -> &str {
        "filesystem"
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn status(&self) -> ConnectionStatus {
        self.status
    }

    async fn initialize(&mut self) -> ConnectorResult<()> {
        let base_path = self.get_base_path()?;

        // Verify base path exists
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .await
                .map_err(|e| ConnectorError::connection(e.to_string()))?;
        }

        self.base_path = Some(base_path);
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> ConnectorResult<()> {
        self.base_path = None;
        self.status = ConnectionStatus::Closed;
        Ok(())
    }

    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse> {
        if self.status != ConnectionStatus::Connected {
            return Err(ConnectorError::connection("File system not connected"));
        }

        let start = Instant::now();

        let result = match request.operation.to_uppercase().as_str() {
            "READ" => {
                if !self.allow_read() {
                    return Err(ConnectorError::validation("Read operations not allowed"));
                }

                let file_path = request
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("File path not specified"))?;

                let full_path = self.validate_path(Path::new(file_path))?;

                match fs::read_to_string(&full_path).await {
                    Ok(content) => json!({
                        "operation": "read",
                        "path": file_path,
                        "size": content.len(),
                        "content": content
                    }),
                    Err(e) => {
                        return Err(ConnectorError::request(format!(
                            "Failed to read file: {}",
                            e
                        )))
                    }
                }
            }
            "WRITE" => {
                if !self.allow_write() {
                    return Err(ConnectorError::validation("Write operations not allowed"));
                }

                let file_path = request
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("File path not specified"))?;

                let content = request
                    .body
                    .as_ref()
                    .and_then(|v| v.get("content"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("File content not specified"))?;

                let full_path = self.validate_path(Path::new(file_path))?;

                match fs::write(&full_path, content).await {
                    Ok(_) => json!({
                        "operation": "write",
                        "path": file_path,
                        "bytes_written": content.len()
                    }),
                    Err(e) => {
                        return Err(ConnectorError::request(format!(
                            "Failed to write file: {}",
                            e
                        )))
                    }
                }
            }
            "DELETE" => {
                if !self.allow_delete() {
                    return Err(ConnectorError::validation("Delete operations not allowed"));
                }

                let file_path = request
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("File path not specified"))?;

                let full_path = self.validate_path(Path::new(file_path))?;

                match fs::remove_file(&full_path).await {
                    Ok(_) => json!({
                        "operation": "delete",
                        "path": file_path
                    }),
                    Err(e) => {
                        return Err(ConnectorError::request(format!(
                            "Failed to delete file: {}",
                            e
                        )))
                    }
                }
            }
            "LIST" => {
                if !self.allow_read() {
                    return Err(ConnectorError::validation("Read operations not allowed"));
                }

                let dir_path = request
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                let full_path = self.validate_path(Path::new(dir_path))?;

                match fs::read_dir(&full_path).await {
                    Ok(mut entries) => {
                        let mut files = Vec::new();
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Ok(metadata) = entry.metadata().await {
                                files.push(json!({
                                    "name": entry.file_name().to_string_lossy().to_string(),
                                    "is_dir": metadata.is_dir(),
                                    "size": metadata.len()
                                }));
                            }
                        }
                        json!({
                            "operation": "list",
                            "path": dir_path,
                            "file_count": files.len(),
                            "files": files
                        })
                    }
                    Err(e) => {
                        return Err(ConnectorError::request(format!(
                            "Failed to list directory: {}",
                            e
                        )))
                    }
                }
            }
            "MKDIR" => {
                if !self.allow_write() {
                    return Err(ConnectorError::validation("Write operations not allowed"));
                }

                let dir_path = request
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConnectorError::request("Directory path not specified"))?;

                let full_path = self.validate_path(Path::new(dir_path))?;

                match fs::create_dir_all(&full_path).await {
                    Ok(_) => json!({
                        "operation": "mkdir",
                        "path": dir_path
                    }),
                    Err(e) => {
                        return Err(ConnectorError::request(format!(
                            "Failed to create directory: {}",
                            e
                        )))
                    }
                }
            }
            _ => {
                return Err(ConnectorError::validation(format!(
                    "Unsupported operation: {}",
                    request.operation
                )))
            }
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ConnectorResponse::new(&request.id, 200)
            .with_body(result)
            .with_execution_time(execution_time_ms))
    }

    async fn health_check(&self) -> ConnectorResult<HealthStatus> {
        if self.status != ConnectionStatus::Connected {
            return Ok(HealthStatus::Unhealthy(format!(
                "File system status: {}",
                self.status
            )));
        }

        if let Some(base_path) = &self.base_path {
            if base_path.exists() && base_path.is_dir() {
                Ok(HealthStatus::Healthy)
            } else {
                Ok(HealthStatus::Unhealthy(
                    "Base path is not accessible".to_string(),
                ))
            }
        } else {
            Ok(HealthStatus::Unhealthy("Base path not initialized".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_connector_creation() {
        let config = ConnectorConfig::new("fs", "filesystem")
            .with_param("base_path", json!("/tmp"));

        let connector = FileSystemConnector::new(config);
        assert_eq!(connector.connector_type(), "filesystem");
        assert_eq!(connector.status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        let config = ConnectorConfig::new("fs", "filesystem")
            .with_param("base_path", json!(base_path));

        let mut connector = FileSystemConnector::new(config);
        connector.base_path = Some(PathBuf::from(base_path));

        // Valid path
        assert!(connector.validate_path(Path::new("test.txt")).is_ok());

        // Escaped path
        assert!(connector.validate_path(Path::new("../../../etc/passwd")).is_err());
    }

    #[test]
    fn test_permission_checks() {
        let config = ConnectorConfig::new("fs", "filesystem")
            .with_param("base_path", json!("/tmp"))
            .with_config("allow_read", json!(true))
            .with_config("allow_write", json!(false))
            .with_config("allow_delete", json!(false));

        let connector = FileSystemConnector::new(config);
        assert!(connector.allow_read());
        assert!(!connector.allow_write());
        assert!(!connector.allow_delete());
    }
}
