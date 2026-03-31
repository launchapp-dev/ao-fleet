use anyhow::{Result, bail};
use ao_fleet_store::FleetStore;
use serde::Serialize;

use crate::cli::handlers::host_sync_all_command::HostSyncAllCommand;
use crate::cli::handlers::host_sync_support::{
    HostSyncOptions, HostSyncResult, sync_host_by_base_url,
};
use crate::cli::handlers::json_printer::print_json;

pub fn host_sync_all(db_path: &str, command: HostSyncAllCommand) -> Result<()> {
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

    let mut results = Vec::new();
    let mut skipped_hosts = Vec::new();
    for host in store.list_hosts()? {
        if !host.address.starts_with("http://") && !host.address.starts_with("https://") {
            skipped_hosts.push(SkippedHost {
                host_id: host.id,
                slug: host.slug,
                address: host.address,
                reason: "host_address_not_http_url".to_string(),
            });
            continue;
        }

        results.push(sync_host_by_base_url(&store, &host.address, &options)?);
    }

    print_json(&HostSyncAllResult {
        host_count: results.len(),
        skipped_count: skipped_hosts.len(),
        register_missing: command.register_missing,
        team_id: command.team_id,
        results,
        skipped_hosts,
    })
}

#[derive(Debug, Serialize)]
struct HostSyncAllResult {
    host_count: usize,
    skipped_count: usize,
    register_missing: bool,
    team_id: Option<String>,
    results: Vec<HostSyncResult>,
    skipped_hosts: Vec<SkippedHost>,
}

#[derive(Debug, Serialize)]
struct SkippedHost {
    host_id: String,
    slug: String,
    address: String,
    reason: String,
}
