use anyhow::Result;

use ao_fleet_core::NewHost;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::host_create_command::HostCreateCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_create(db_path: &str, command: HostCreateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let host = store.create_host(NewHost {
        slug: command.slug,
        name: command.name,
        address: command.address,
        platform: command.platform,
        status: command.status,
        capacity_slots: command.capacity_slots,
    })?;
    print_json(&host)
}
