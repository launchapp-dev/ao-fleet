use clap::{Parser, Subcommand};

use crate::cli::handlers::daemon_reconcile_command::DaemonReconcileCommand;
use crate::cli::handlers::db_init_command::DbInitCommand;
use crate::cli::handlers::mcp_list_command::McpListCommand;
use crate::cli::handlers::project_create_command::ProjectCreateCommand;
use crate::cli::handlers::project_list_command::ProjectListCommand;
use crate::cli::handlers::schedule_create_command::ScheduleCreateCommand;
use crate::cli::handlers::schedule_list_command::ScheduleListCommand;
use crate::cli::handlers::team_create_command::TeamCreateCommand;
use crate::cli::handlers::team_list_command::TeamListCommand;

#[derive(Debug, Parser)]
#[command(name = "ao-fleet")]
#[command(about = "Fleet control plane for AO teams")]
pub struct RootCommand {
    #[arg(long, global = true, default_value = "ao-fleet.db")]
    pub db_path: String,

    #[command(subcommand)]
    pub command: CommandGroup,
}

#[derive(Debug, Subcommand)]
pub enum CommandGroup {
    DbInit(DbInitCommand),
    TeamCreate(TeamCreateCommand),
    TeamList(TeamListCommand),
    ProjectCreate(ProjectCreateCommand),
    ProjectList(ProjectListCommand),
    ScheduleCreate(ScheduleCreateCommand),
    ScheduleList(ScheduleListCommand),
    DaemonReconcile(DaemonReconcileCommand),
    McpList(McpListCommand),
}
