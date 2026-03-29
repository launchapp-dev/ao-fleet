use anyhow::Result;
use ao_fleet_store::{FleetOverviewQuery, FleetStore};

use crate::cli::handlers::founder_overview_command::FounderOverviewCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn founder_overview(db_path: &str, command: FounderOverviewCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let overview = store.founder_overview(FleetOverviewQuery {
        team_id: command.team_id,
        ..FleetOverviewQuery::default()
    })?;
    print_json(&overview)
}
