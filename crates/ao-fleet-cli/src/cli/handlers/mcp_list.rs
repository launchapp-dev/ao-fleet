use anyhow::Result;

use ao_fleet_mcp::FleetMcpSurface;

use crate::cli::handlers::mcp_list_command::McpListCommand;

pub fn mcp_list(_command: McpListCommand) -> Result<()> {
    let surface = FleetMcpSurface::default();
    println!("{}", surface.to_pretty_json()?);
    Ok(())
}
