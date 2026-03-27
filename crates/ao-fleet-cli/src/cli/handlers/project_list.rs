use anyhow::Result;

use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_list_command::ProjectListCommand;

pub fn project_list(db_path: &str, _command: ProjectListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let projects = store.list_projects(None)?;
    print_json(&projects)
}
