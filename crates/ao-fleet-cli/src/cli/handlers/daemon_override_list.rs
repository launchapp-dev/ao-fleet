use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::daemon_override_list_command::DaemonOverrideListCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn daemon_override_list(db_path: &str, _command: DaemonOverrideListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    print_json(&store.list_daemon_overrides()?)
}
