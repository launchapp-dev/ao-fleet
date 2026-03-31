use anyhow::{Result, bail};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::host_sync_command::HostSyncCommand;
use crate::cli::handlers::host_sync_support::{HostSyncOptions, sync_host_by_base_url};
use crate::cli::handlers::json_printer::print_json;

pub fn host_sync(db_path: &str, command: HostSyncCommand) -> Result<()> {
    if command.register_missing && command.team_id.is_none() {
        bail!("--team-id is required when --register-missing is set");
    }

    let store = FleetStore::open(db_path)?;
    let auth_token =
        command.auth_token.clone().or_else(|| std::env::var("AO_FLEET_HOSTD_AUTH_TOKEN").ok());
    let options = HostSyncOptions {
        auth_token,
        register_missing: command.register_missing,
        team_id: command.team_id.clone(),
        assignment_source: command.assignment_source.clone(),
    };

    let result = sync_host_by_base_url(&store, &command.base_url, &options)?;
    print_json(&result)
}
