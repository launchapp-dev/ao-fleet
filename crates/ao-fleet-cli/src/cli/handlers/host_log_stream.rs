use anyhow::Result;
use ao_fleet_ao::AoHostdWsClient;

use crate::cli::handlers::host_log_stream_command::HostLogStreamCommand;

pub fn host_log_stream(_db_path: &str, command: HostLogStreamCommand) -> Result<()> {
    let auth_token =
        command.auth_token.clone().or_else(|| std::env::var("AO_FLEET_HOSTD_AUTH_TOKEN").ok());
    let client = AoHostdWsClient::new(command.base_url, auth_token)?;

    client.stream_logs(
        command.project_id.as_deref(),
        command.after_seq,
        command.tail,
        command.cat.as_deref(),
        command.level.as_deref(),
        command.workflow.as_deref(),
        command.run.as_deref(),
        |event| {
            println!("{}", serde_json::to_string(&event)?);
            Ok(())
        },
    )
}
