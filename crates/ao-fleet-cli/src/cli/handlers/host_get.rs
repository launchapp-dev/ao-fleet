use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::host_get_command::HostGetCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_get(db_path: &str, command: HostGetCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let host =
        store.get_host(&command.id)?.ok_or_else(|| anyhow!("host not found: {}", command.id))?;
    print_json(&host)
}
