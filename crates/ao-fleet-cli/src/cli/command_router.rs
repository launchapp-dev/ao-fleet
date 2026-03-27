use anyhow::Result;

use crate::cli::handlers::audit_list::audit_list;
use crate::cli::handlers::daemon_reconcile::daemon_reconcile;
use crate::cli::handlers::db_init::db_init;
use crate::cli::handlers::mcp_list::mcp_list;
use crate::cli::handlers::mcp_serve::mcp_serve;
use crate::cli::handlers::project_create::project_create;
use crate::cli::handlers::project_delete::project_delete;
use crate::cli::handlers::project_get::project_get;
use crate::cli::handlers::project_list::project_list;
use crate::cli::handlers::project_update::project_update;
use crate::cli::handlers::schedule_create::schedule_create;
use crate::cli::handlers::schedule_delete::schedule_delete;
use crate::cli::handlers::schedule_get::schedule_get;
use crate::cli::handlers::schedule_list::schedule_list;
use crate::cli::handlers::schedule_update::schedule_update;
use crate::cli::handlers::team_create::team_create;
use crate::cli::handlers::team_delete::team_delete;
use crate::cli::handlers::team_get::team_get;
use crate::cli::handlers::team_list::team_list;
use crate::cli::handlers::team_update::team_update;
use crate::cli::root_command::{CommandGroup, RootCommand};

pub fn route_command(root: RootCommand) -> Result<()> {
    match root.command {
        CommandGroup::DbInit(command) => db_init(&root.db_path, command),
        CommandGroup::AuditList(command) => audit_list(&root.db_path, command),
        CommandGroup::TeamCreate(command) => team_create(&root.db_path, command),
        CommandGroup::TeamGet(command) => team_get(&root.db_path, command),
        CommandGroup::TeamList(command) => team_list(&root.db_path, command),
        CommandGroup::TeamUpdate(command) => team_update(&root.db_path, command),
        CommandGroup::TeamDelete(command) => team_delete(&root.db_path, command),
        CommandGroup::ProjectCreate(command) => project_create(&root.db_path, command),
        CommandGroup::ProjectGet(command) => project_get(&root.db_path, command),
        CommandGroup::ProjectList(command) => project_list(&root.db_path, command),
        CommandGroup::ProjectUpdate(command) => project_update(&root.db_path, command),
        CommandGroup::ProjectDelete(command) => project_delete(&root.db_path, command),
        CommandGroup::ScheduleCreate(command) => schedule_create(&root.db_path, command),
        CommandGroup::ScheduleGet(command) => schedule_get(&root.db_path, command),
        CommandGroup::ScheduleList(command) => schedule_list(&root.db_path, command),
        CommandGroup::ScheduleUpdate(command) => schedule_update(&root.db_path, command),
        CommandGroup::ScheduleDelete(command) => schedule_delete(&root.db_path, command),
        CommandGroup::DaemonReconcile(command) => daemon_reconcile(&root.db_path, command),
        CommandGroup::McpList(command) => mcp_list(command),
        CommandGroup::McpServe(command) => mcp_serve(&root.db_path, command),
    }
}
