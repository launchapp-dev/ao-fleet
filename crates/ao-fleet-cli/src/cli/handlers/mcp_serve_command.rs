use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Run the fleet MCP server over stdio")]
pub struct McpServeCommand;
