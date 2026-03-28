use anyhow::Result;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_events_command::ProjectEventsCommand;
use crate::cli::handlers::project_ops_support::{list_project_events, stream_project_events};

pub fn project_events(db_path: &str, command: ProjectEventsCommand) -> Result<()> {
    if command.follow {
        return stream_project_events(db_path, &command);
    }

    let value = list_project_events(db_path, &command)?;
    print_json(&value)
}
