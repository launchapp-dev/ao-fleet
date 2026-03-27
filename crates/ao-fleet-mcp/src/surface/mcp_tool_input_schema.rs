use serde::{Deserialize, Serialize};

use super::mcp_tool_property::McpToolProperty;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolInputSchema {
    pub description: String,
    pub properties: Vec<McpToolProperty>,
    pub required: Vec<String>,
    pub additional_properties: bool,
}
