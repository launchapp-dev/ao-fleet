use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::host_list_command::HostListCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_list(db_path: &str, _command: HostListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    print_json(&store.list_hosts()?)
}
