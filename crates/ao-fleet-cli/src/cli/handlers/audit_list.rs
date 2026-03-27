use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::audit_list_command::AuditListCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn audit_list(db_path: &str, command: AuditListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let events = store.list_audit_events(command.team_id.as_deref(), command.limit)?;
    print_json(&events)
}
