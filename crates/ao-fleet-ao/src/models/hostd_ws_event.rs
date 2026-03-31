use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostdWsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}
