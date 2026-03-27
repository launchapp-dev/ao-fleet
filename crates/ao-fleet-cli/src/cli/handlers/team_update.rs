use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::team_update_command::TeamUpdateCommand;

pub fn team_update(db_path: &str, command: TeamUpdateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let mut team =
        store.get_team(&command.id)?.ok_or_else(|| anyhow!("team not found: {}", command.id))?;

    if let Some(value) = command.slug {
        team.slug = value;
    }
    if let Some(value) = command.name {
        team.name = value;
    }
    if let Some(value) = command.mission {
        team.mission = value;
    }
    if let Some(value) = command.ownership {
        team.ownership = value;
    }
    if let Some(value) = command.business_priority {
        team.business_priority = value;
    }
    team.updated_at = Utc::now();

    let team = store.update_team(team)?;
    print_json(&team)
}
