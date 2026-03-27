use anyhow::Result;

use ao_fleet_core::NewTeam;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::team_create_command::TeamCreateCommand;

pub fn team_create(db_path: &str, command: TeamCreateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let team = store.create_team(NewTeam {
        slug: command.slug,
        name: command.name,
        mission: command.mission,
        ownership: command.ownership,
        business_priority: command.business_priority,
    })?;
    print_json(&team)
}
