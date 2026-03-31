use anyhow::Result;
use ao_fleet_ao::AoHostdClient;

use crate::cli::handlers::host_logs_command::HostLogsCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_logs(_db_path: &str, command: HostLogsCommand) -> Result<()> {
    let auth_token =
        command.auth_token.clone().or_else(|| std::env::var("AO_FLEET_HOSTD_AUTH_TOKEN").ok());
    let client = AoHostdClient::new(command.base_url, auth_token)?;
    let response = client.list_logs(
        command.project_id.as_deref(),
        command.after_seq,
        Some(command.limit),
        command.cat.as_deref(),
        command.level.as_deref(),
        command.workflow.as_deref(),
        command.run.as_deref(),
    )?;

    print_json(&response)
}
