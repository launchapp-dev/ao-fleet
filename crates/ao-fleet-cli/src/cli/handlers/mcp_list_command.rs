use clap::Args;

#[derive(Debug, Args)]
#[command(about = "List available MCP tools exposed by this fleet server")]
pub struct McpListCommand;
