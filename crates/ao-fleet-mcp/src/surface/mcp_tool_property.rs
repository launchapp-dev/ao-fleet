use serde::{Deserialize, Serialize};

use super::mcp_tool_value_kind::McpToolValueKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolProperty {
    pub name: String,
    pub kind: McpToolValueKind,
    pub description: String,
    pub required: bool,
    pub enum_values: Vec<String>,
    pub example: Option<serde_json::Value>,
}
