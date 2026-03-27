use anyhow::Result;

use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::team_list_command::TeamListCommand;

pub fn team_list(db_path: &str, _command: TeamListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let teams = store.list_teams()?;
    print_json(&teams)
}
