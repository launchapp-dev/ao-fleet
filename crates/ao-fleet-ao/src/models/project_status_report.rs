use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AoCommandError;
use crate::models::daemon_state::DaemonState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectStatusReport {
    pub project_root: String,
    pub generated_at: DateTime<Utc>,
    pub daemon_available: bool,
    pub daemon_state: DaemonState,
    pub daemon_running: bool,
    pub runner_connected: bool,
    pub runner_pid: Option<u32>,
    pub active_agents_available: bool,
    pub active_agents_count: usize,
    pub task_total: usize,
    pub task_done: usize,
    pub task_in_progress: usize,
    pub task_ready: usize,
    pub task_blocked: usize,
}

impl ProjectStatusReport {
    pub(crate) fn from_cli_value(
        project_root: &Path,
        value: serde_json::Value,
    ) -> Result<Self, AoCommandError> {
        let generated_at = parse_datetime(project_root, &value, "generated_at")?;
        let daemon = value
            .get("daemon")
            .ok_or_else(|| invalid_response("missing daemon section", &value, project_root))?;
        let active_agents = value.get("active_agents").ok_or_else(|| {
            invalid_response("missing active_agents section", &value, project_root)
        })?;
        let task_summary = value.get("task_summary").ok_or_else(|| {
            invalid_response("missing task_summary section", &value, project_root)
        })?;

        Ok(Self {
            project_root: project_root.to_string_lossy().to_string(),
            generated_at,
            daemon_available: daemon
                .get("available")
                .and_then(|entry| entry.as_bool())
                .unwrap_or(false),
            daemon_state: daemon
                .get("status")
                .and_then(|entry| entry.as_str())
                .map(DaemonState::from_cli_value)
                .unwrap_or(DaemonState::Unknown("unknown".to_string())),
            daemon_running: daemon
                .get("running")
                .and_then(|entry| entry.as_bool())
                .unwrap_or(false),
            runner_connected: daemon
                .get("runner_connected")
                .and_then(|entry| entry.as_bool())
                .unwrap_or(false),
            runner_pid: daemon
                .get("runner_pid")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as u32),
            active_agents_available: active_agents
                .get("available")
                .and_then(|entry| entry.as_bool())
                .unwrap_or(false),
            active_agents_count: active_agents
                .get("count")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
            task_total: task_summary
                .get("total")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
            task_done: task_summary
                .get("done")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
            task_in_progress: task_summary
                .get("in_progress")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
            task_ready: task_summary
                .get("ready")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
            task_blocked: task_summary
                .get("blocked")
                .and_then(|entry| entry.as_u64())
                .map(|value| value as usize)
                .unwrap_or(0),
        })
    }
}

fn parse_datetime(
    project_root: &Path,
    value: &serde_json::Value,
    key: &str,
) -> Result<DateTime<Utc>, AoCommandError> {
    let raw = value
        .get(key)
        .and_then(|entry| entry.as_str())
        .ok_or_else(|| invalid_response(&format!("missing {key}"), value, project_root))?;
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| invalid_response(&format!("invalid {key}: {error}"), value, project_root))
}

fn invalid_response(
    message: &str,
    value: &serde_json::Value,
    project_root: &Path,
) -> AoCommandError {
    AoCommandError::invalid_response(
        PathBuf::from("ao"),
        project_root.to_path_buf(),
        message.to_string(),
        value.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_project_status_payload() {
        let payload = serde_json::json!({
            "schema": "ao.status.v1",
            "project_root": "/tmp/project",
            "generated_at": "2026-03-27T00:00:00Z",
            "daemon": {
                "available": true,
                "status": "running",
                "running": true,
                "runner_connected": true,
                "runner_pid": 1234
            },
            "active_agents": {
                "available": true,
                "count": 2
            },
            "task_summary": {
                "total": 10,
                "done": 4,
                "in_progress": 2,
                "ready": 3,
                "blocked": 1
            }
        });

        let report = ProjectStatusReport::from_cli_value(Path::new("/tmp/project"), payload)
            .expect("project status parses");

        assert_eq!(report.project_root, "/tmp/project");
        assert_eq!(report.daemon_state, DaemonState::Running);
        assert_eq!(report.runner_pid, Some(1234));
        assert_eq!(report.task_blocked, 1);
    }
}
