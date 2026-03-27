use anyhow::Result;

use crate::cli::handlers::daemon_reconcile::daemon_reconcile;
use crate::cli::handlers::db_init::db_init;
use crate::cli::handlers::mcp_list::mcp_list;
use crate::cli::handlers::project_create::project_create;
use crate::cli::handlers::project_list::project_list;
use crate::cli::handlers::schedule_create::schedule_create;
use crate::cli::handlers::schedule_list::schedule_list;
use crate::cli::handlers::team_create::team_create;
use crate::cli::handlers::team_list::team_list;
use crate::cli::root_command::{CommandGroup, RootCommand};

pub fn route_command(root: RootCommand) -> Result<()> {
    match root.command {
        CommandGroup::DbInit(command) => db_init(&root.db_path, command),
        CommandGroup::TeamCreate(command) => team_create(&root.db_path, command),
        CommandGroup::TeamList(command) => team_list(&root.db_path, command),
        CommandGroup::ProjectCreate(command) => project_create(&root.db_path, command),
        CommandGroup::ProjectList(command) => project_list(&root.db_path, command),
        CommandGroup::ScheduleCreate(command) => schedule_create(&root.db_path, command),
        CommandGroup::ScheduleList(command) => schedule_list(&root.db_path, command),
        CommandGroup::DaemonReconcile(command) => daemon_reconcile(&root.db_path, command),
        CommandGroup::McpList(command) => mcp_list(command),
    }
}
