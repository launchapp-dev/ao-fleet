use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::team_get_command::TeamGetCommand;

pub fn team_get(db_path: &str, command: TeamGetCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let team =
        store.get_team(&command.id)?.ok_or_else(|| anyhow!("team not found: {}", command.id))?;
    print_json(&team)
}
