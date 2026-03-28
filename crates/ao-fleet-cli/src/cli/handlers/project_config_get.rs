use anyhow::Result;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_config_get_command::ProjectConfigGetCommand;
use crate::cli::handlers::project_ops_support::load_project_config_value;

pub fn project_config_get(db_path: &str, command: ProjectConfigGetCommand) -> Result<()> {
    let value = load_project_config_value(db_path, &command.project_root)?;
    print_json(&value)
}
