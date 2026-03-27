use anyhow::Result;

use ao_fleet_core::NewProject;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_create_command::ProjectCreateCommand;

pub fn project_create(db_path: &str, command: ProjectCreateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let project = store.create_project(NewProject {
        team_id: command.team_id,
        slug: command.slug,
        root_path: command.root_path,
        ao_project_root: command.ao_project_root,
        default_branch: command.default_branch,
        remote_url: command.remote_url,
        enabled: command.enabled,
    })?;
    print_json(&project)
}
