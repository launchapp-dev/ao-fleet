use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_host_list_command::ProjectHostListCommand;

pub fn project_host_list(db_path: &str, _command: ProjectHostListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    print_json(&store.list_project_host_placements()?)
}
