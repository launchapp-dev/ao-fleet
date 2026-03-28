use anyhow::Result;
use ao_fleet_store::{FleetOverviewQuery, FleetStore};

use crate::cli::handlers::fleet_overview_command::FleetOverviewCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn fleet_overview(db_path: &str, command: FleetOverviewCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let overview = store.fleet_overview(FleetOverviewQuery {
        team_id: command.team_id,
        ..FleetOverviewQuery::default()
    })?;
    print_json(&overview)
}
