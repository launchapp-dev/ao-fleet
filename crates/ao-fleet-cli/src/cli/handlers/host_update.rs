use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::host_update_command::HostUpdateCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_update(db_path: &str, command: HostUpdateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let mut host =
        store.get_host(&command.id)?.ok_or_else(|| anyhow!("host not found: {}", command.id))?;

    if let Some(value) = command.slug {
        host.slug = value;
    }
    if let Some(value) = command.name {
        host.name = value;
    }
    if let Some(value) = command.address {
        host.address = value;
    }
    if let Some(value) = command.platform {
        host.platform = value;
    }
    if let Some(value) = command.status {
        host.status = value;
    }
    if let Some(value) = command.capacity_slots {
        host.capacity_slots = value;
    }
    host.updated_at = Utc::now();

    print_json(&store.update_host(host)?)
}
