use anyhow::Result;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_ao_json_command::ProjectAoJsonCommand;
use crate::cli::handlers::project_ops_support::execute_project_json_command;

pub fn project_ao_json(db_path: &str, command: ProjectAoJsonCommand) -> Result<()> {
    let value = execute_project_json_command(db_path, &command.project_root, &command.args)?;
    print_json(&value)
}
