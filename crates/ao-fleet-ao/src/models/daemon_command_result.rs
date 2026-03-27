use serde::{Deserialize, Serialize};

use crate::models::daemon_state::DaemonState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DaemonCommandResult {
    pub command: String,
    pub message: Option<String>,
    pub daemon_pid: Option<u32>,
    pub state: Option<DaemonState>,
    pub raw: serde_json::Value,
}

impl DaemonCommandResult {
    pub fn from_cli_value(command: impl Into<String>, value: serde_json::Value) -> Self {
        let command = command.into();
        let message = extract_string(&value, "message");
        let daemon_pid = extract_u32(&value, "daemon_pid");
        let state =
            extract_string(&value, "state").map(|state| DaemonState::from_cli_value(&state));

        Self { command, message, daemon_pid, state, raw: value }
    }
}

fn extract_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|entry| entry.as_str()).map(String::from)
}

fn extract_u32(value: &serde_json::Value, key: &str) -> Option<u32> {
    value.get(key).and_then(|entry| entry.as_u64()).map(|entry| entry as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_state_from_raw_result() {
        let result = DaemonCommandResult::from_cli_value(
            "pause",
            serde_json::json!({"message": "paused", "state": "paused"}),
        );

        assert_eq!(result.command, "pause");
        assert_eq!(result.message.as_deref(), Some("paused"));
        assert_eq!(result.state, Some(DaemonState::Paused));
    }
}
