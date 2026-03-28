use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::daemon_override_clear_command::DaemonOverrideClearCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn daemon_override_clear(db_path: &str, command: DaemonOverrideClearCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let cleared = store.clear_daemon_override(&command.team_id)?;
    print_json(&serde_json::json!({
        "team_id": command.team_id,
        "cleared": cleared,
    }))
}
