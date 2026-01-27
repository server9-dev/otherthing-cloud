//! Development Activity Logger
//!
//! Logs all development activities (commands, tool uses, responses) in a format
//! compatible with BPMN 2.0 and DoDAF 2.02 specifications for future analysis.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// Activity type classification aligned with BPMN
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// User task - manual interaction
    UserTask,
    /// Service task - automated tool execution
    ServiceTask,
    /// Script task - code execution
    ScriptTask,
    /// Send task - output/response generation
    SendTask,
    /// Receive task - input/data reception
    ReceiveTask,
    /// Manual task - manual code changes
    ManualTask,
}

/// DoDAF Operational Activity Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalActivity {
    /// Activity identifier
    pub id: String,
    /// Activity name
    pub name: String,
    /// Activity type per BPMN classification
    pub activity_type: ActivityType,
    /// Timestamp when activity started
    pub start_time: DateTime<Utc>,
    /// Timestamp when activity ended
    pub end_time: Option<DateTime<Utc>>,
    /// Performer (agent, tool, human)
    pub performer: String,
    /// Activity description
    pub description: String,
    /// Input data or parameters
    pub inputs: serde_json::Value,
    /// Output data or results
    pub outputs: Option<serde_json::Value>,
    /// Parent activity ID for nested activities
    pub parent_id: Option<String>,
    /// Success status
    pub success: Option<bool>,
    /// Error information if any
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Process flow representing sequence between activities (BPMN Sequence Flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFlow {
    /// Flow identifier
    pub id: String,
    /// Source activity ID
    pub source_activity_id: String,
    /// Target activity ID
    pub target_activity_id: String,
    /// Flow condition if any (for gateways)
    pub condition: Option<String>,
    /// Flow name/label
    pub name: Option<String>,
}

/// Complete development log structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentLog {
    /// Log metadata
    pub log_metadata: LogMetadata,
    /// Operational activities (DoDAF OV-5)
    pub operational_activities: Vec<OperationalActivity>,
    /// Process flows (BPMN sequence flows)
    pub process_flows: Vec<ProcessFlow>,
    /// Simple activity list for backward compatibility
    pub activities: Vec<SimpleActivity>,
}

/// Log metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub project: String,
    pub description: String,
    pub standards: Standards,
}

/// Standards compliance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standards {
    pub bpmn_version: String,
    pub dodaf_version: String,
}

/// Simple activity for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleActivity {
    pub timestamp: DateTime<Utc>,
    pub activity_type: String,
    pub description: String,
    pub details: serde_json::Value,
}

/// Development activity logger
pub struct DevLogger {
    log_path: String,
    current_activity_id: usize,
    current_flow_id: usize,
}

impl DevLogger {
    /// Create a new logger
    pub fn new(log_path: impl Into<String>) -> Self {
        Self { log_path: log_path.into(), current_activity_id: 1, current_flow_id: 1 }
    }

    /// Initialize the log file if it doesn't exist or is empty
    pub fn init(&self) -> Result<()> {
        let path = Path::new(&self.log_path);
        let needs_init = !path.exists() || {
            // Check if file is empty
            std::fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true)
        };

        if needs_init {
            let log = DevelopmentLog {
                log_metadata: LogMetadata {
                    version: "1.0.0".to_string(),
                    created_at: Utc::now(),
                    project: "abcdodaf".to_string(),
                    description:
                        "Development activity log for BPMN and DoDAF compliant process modeling"
                            .to_string(),
                    standards: Standards {
                        bpmn_version: "2.0".to_string(),
                        dodaf_version: "2.02".to_string(),
                    },
                },
                operational_activities: Vec::new(),
                process_flows: Vec::new(),
                activities: Vec::new(),
            };
            self.write_log(&log)?;
        }
        Ok(())
    }

    /// Load the current log
    fn load_log(&self) -> Result<DevelopmentLog> {
        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);
        let log = serde_json::from_reader(reader)?;
        Ok(log)
    }

    /// Write the log to file
    fn write_log(&self, log: &DevelopmentLog) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.log_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, log)?;
        writer.flush()?;
        Ok(())
    }

    /// Log a tool use activity
    pub fn log_tool_use(
        &mut self,
        tool_name: &str,
        parameters: serde_json::Value,
        result: Option<serde_json::Value>,
        success: bool,
    ) -> Result<String> {
        let mut log = self.load_log()?;

        let activity_id = format!("activity_{}", self.current_activity_id);
        self.current_activity_id += 1;

        let activity = OperationalActivity {
            id: activity_id.clone(),
            name: format!("Execute {}", tool_name),
            activity_type: ActivityType::ServiceTask,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            performer: format!("Tool::{}", tool_name),
            description: format!("Execute {} tool", tool_name),
            inputs: parameters,
            outputs: result,
            parent_id: None,
            success: Some(success),
            error: None,
            metadata: serde_json::json!({
                "tool_name": tool_name,
            }),
        };

        log.operational_activities.push(activity);

        // Add simple activity for backward compatibility
        log.activities.push(SimpleActivity {
            timestamp: Utc::now(),
            activity_type: "tool_use".to_string(),
            description: format!("Used tool: {}", tool_name),
            details: serde_json::json!({
                "tool": tool_name,
                "success": success,
            }),
        });

        self.write_log(&log)?;
        Ok(activity_id)
    }

    /// Log a command execution
    pub fn log_command(
        &mut self,
        command: &str,
        output: Option<&str>,
        exit_code: i32,
    ) -> Result<String> {
        let mut log = self.load_log()?;

        let activity_id = format!("activity_{}", self.current_activity_id);
        self.current_activity_id += 1;

        let activity = OperationalActivity {
            id: activity_id.clone(),
            name: format!("Execute command"),
            activity_type: ActivityType::ScriptTask,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            performer: "System::Shell".to_string(),
            description: format!("Execute shell command: {}", command),
            inputs: serde_json::json!({
                "command": command,
            }),
            outputs: output.map(|o| {
                serde_json::json!({
                    "output": o,
                    "exit_code": exit_code,
                })
            }),
            parent_id: None,
            success: Some(exit_code == 0),
            error: if exit_code != 0 {
                Some(format!("Command failed with exit code {}", exit_code))
            } else {
                None
            },
            metadata: serde_json::json!({
                "command_type": "shell",
            }),
        };

        log.operational_activities.push(activity);

        log.activities.push(SimpleActivity {
            timestamp: Utc::now(),
            activity_type: "command".to_string(),
            description: format!("Executed: {}", command),
            details: serde_json::json!({
                "command": command,
                "exit_code": exit_code,
            }),
        });

        self.write_log(&log)?;
        Ok(activity_id)
    }

    /// Log an AI response generation
    pub fn log_response(&mut self, prompt_summary: &str, response_summary: &str) -> Result<String> {
        let mut log = self.load_log()?;

        let activity_id = format!("activity_{}", self.current_activity_id);
        self.current_activity_id += 1;

        let activity = OperationalActivity {
            id: activity_id.clone(),
            name: "Generate AI Response".to_string(),
            activity_type: ActivityType::SendTask,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            performer: "AI::Claude".to_string(),
            description: "Generate response to user query".to_string(),
            inputs: serde_json::json!({
                "prompt_summary": prompt_summary,
            }),
            outputs: Some(serde_json::json!({
                "response_summary": response_summary,
            })),
            parent_id: None,
            success: Some(true),
            error: None,
            metadata: serde_json::json!({}),
        };

        log.operational_activities.push(activity);

        log.activities.push(SimpleActivity {
            timestamp: Utc::now(),
            activity_type: "response".to_string(),
            description: "Generated AI response".to_string(),
            details: serde_json::json!({
                "prompt_summary": prompt_summary,
                "response_summary": response_summary,
            }),
        });

        self.write_log(&log)?;
        Ok(activity_id)
    }

    /// Create a process flow between two activities
    pub fn log_flow(
        &mut self,
        source_id: &str,
        target_id: &str,
        condition: Option<String>,
        name: Option<String>,
    ) -> Result<String> {
        let mut log = self.load_log()?;

        let flow_id = format!("flow_{}", self.current_flow_id);
        self.current_flow_id += 1;

        let flow = ProcessFlow {
            id: flow_id.clone(),
            source_activity_id: source_id.to_string(),
            target_activity_id: target_id.to_string(),
            condition,
            name,
        };

        log.process_flows.push(flow);
        self.write_log(&log)?;
        Ok(flow_id)
    }

    /// Get activity statistics
    pub fn get_stats(&self) -> Result<LogStats> {
        let log = self.load_log()?;

        let total_activities = log.operational_activities.len();
        let successful =
            log.operational_activities.iter().filter(|a| a.success == Some(true)).count();
        let failed = log.operational_activities.iter().filter(|a| a.success == Some(false)).count();

        let mut activity_types = std::collections::HashMap::new();
        for activity in &log.operational_activities {
            *activity_types.entry(format!("{:?}", activity.activity_type)).or_insert(0) += 1;
        }

        Ok(LogStats {
            total_activities,
            successful,
            failed,
            total_flows: log.process_flows.len(),
            activity_types,
        })
    }
}

/// Log statistics
#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_activities: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_flows: usize,
    pub activity_types: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_logger_init() {
        let temp = NamedTempFile::new().unwrap();
        let logger = DevLogger::new(temp.path().to_str().unwrap());
        logger.init().unwrap();

        let log = logger.load_log().unwrap();
        assert_eq!(log.log_metadata.project, "abcdodaf");
    }

    #[test]
    fn test_log_tool_use() {
        let temp = NamedTempFile::new().unwrap();
        let mut logger = DevLogger::new(temp.path().to_str().unwrap());
        logger.init().unwrap();

        let params = serde_json::json!({"param": "value"});
        let result = serde_json::json!({"result": "success"});

        let activity_id = logger.log_tool_use("TestTool", params, Some(result), true).unwrap();
        assert!(activity_id.starts_with("activity_"));

        let stats = logger.get_stats().unwrap();
        assert_eq!(stats.total_activities, 1);
        assert_eq!(stats.successful, 1);
    }
}
