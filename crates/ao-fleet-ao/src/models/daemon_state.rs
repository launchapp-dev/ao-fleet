use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum DaemonState {
    Running,
    Paused,
    Stopped,
    Crashed,
    Unknown(String),
}

impl DaemonState {
    pub fn from_cli_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "stopped" => Self::Stopped,
            "crashed" => Self::Crashed,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl From<DaemonState> for String {
    fn from(value: DaemonState) -> Self {
        match value {
            DaemonState::Running => "running".to_string(),
            DaemonState::Paused => "paused".to_string(),
            DaemonState::Stopped => "stopped".to_string(),
            DaemonState::Crashed => "crashed".to_string(),
            DaemonState::Unknown(other) => other,
        }
    }
}

impl TryFrom<String> for DaemonState {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(DaemonState::from_cli_value(&value))
    }
}
