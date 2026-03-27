use serde::{Deserialize, Serialize};

use super::mcp_tool_input_schema::McpToolInputSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    pub description: String,
    pub input_schema: McpToolInputSchema,
    pub tags: Vec<String>,
}
