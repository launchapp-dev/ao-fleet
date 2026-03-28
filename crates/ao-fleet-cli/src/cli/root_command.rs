use clap::{Parser, Subcommand};

use crate::cli::handlers::audit_list_command::AuditListCommand;
use crate::cli::handlers::config_snapshot_export_command::ConfigSnapshotExportCommand;
use crate::cli::handlers::config_snapshot_import_command::ConfigSnapshotImportCommand;
use crate::cli::handlers::daemon_override_clear_command::DaemonOverrideClearCommand;
use crate::cli::handlers::daemon_override_list_command::DaemonOverrideListCommand;
use crate::cli::handlers::daemon_override_upsert_command::DaemonOverrideUpsertCommand;
use crate::cli::handlers::daemon_reconcile_command::DaemonReconcileCommand;
use crate::cli::handlers::daemon_status_command::DaemonStatusCommand;
use crate::cli::handlers::db_init_command::DbInitCommand;
use crate::cli::handlers::fleet_overview_command::FleetOverviewCommand;
use crate::cli::handlers::host_create_command::HostCreateCommand;
use crate::cli::handlers::host_delete_command::HostDeleteCommand;
use crate::cli::handlers::host_get_command::HostGetCommand;
use crate::cli::handlers::host_list_command::HostListCommand;
use crate::cli::handlers::host_update_command::HostUpdateCommand;
use crate::cli::handlers::knowledge_document_create_command::KnowledgeDocumentCreateCommand;
use crate::cli::handlers::knowledge_document_list_command::KnowledgeDocumentListCommand;
use crate::cli::handlers::knowledge_fact_create_command::KnowledgeFactCreateCommand;
use crate::cli::handlers::knowledge_fact_list_command::KnowledgeFactListCommand;
use crate::cli::handlers::knowledge_search_command::KnowledgeSearchCommand;
use crate::cli::handlers::knowledge_source_list_command::KnowledgeSourceListCommand;
use crate::cli::handlers::knowledge_source_upsert_command::KnowledgeSourceUpsertCommand;
use crate::cli::handlers::mcp_list_command::McpListCommand;
use crate::cli::handlers::mcp_serve_command::McpServeCommand;
use crate::cli::handlers::project_ao_json_command::ProjectAoJsonCommand;
use crate::cli::handlers::project_config_get_command::ProjectConfigGetCommand;
use crate::cli::handlers::project_create_command::ProjectCreateCommand;
use crate::cli::handlers::project_delete_command::ProjectDeleteCommand;
use crate::cli::handlers::project_discover_command::ProjectDiscoverCommand;
use crate::cli::handlers::project_events_command::ProjectEventsCommand;
use crate::cli::handlers::project_get_command::ProjectGetCommand;
use crate::cli::handlers::project_host_assign_command::ProjectHostAssignCommand;
use crate::cli::handlers::project_host_clear_command::ProjectHostClearCommand;
use crate::cli::handlers::project_host_list_command::ProjectHostListCommand;
use crate::cli::handlers::project_list_command::ProjectListCommand;
use crate::cli::handlers::project_update_command::ProjectUpdateCommand;
use crate::cli::handlers::schedule_create_command::ScheduleCreateCommand;
use crate::cli::handlers::schedule_delete_command::ScheduleDeleteCommand;
use crate::cli::handlers::schedule_get_command::ScheduleGetCommand;
use crate::cli::handlers::schedule_list_command::ScheduleListCommand;
use crate::cli::handlers::schedule_update_command::ScheduleUpdateCommand;
use crate::cli::handlers::team_create_command::TeamCreateCommand;
use crate::cli::handlers::team_delete_command::TeamDeleteCommand;
use crate::cli::handlers::team_get_command::TeamGetCommand;
use crate::cli::handlers::team_list_command::TeamListCommand;
use crate::cli::handlers::team_update_command::TeamUpdateCommand;

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
    AuditList(AuditListCommand),
    ConfigSnapshotExport(ConfigSnapshotExportCommand),
    ConfigSnapshotImport(ConfigSnapshotImportCommand),
    FleetOverview(FleetOverviewCommand),
    HostCreate(HostCreateCommand),
    HostGet(HostGetCommand),
    HostList(HostListCommand),
    HostUpdate(HostUpdateCommand),
    HostDelete(HostDeleteCommand),
    TeamCreate(TeamCreateCommand),
    TeamGet(TeamGetCommand),
    TeamList(TeamListCommand),
    TeamUpdate(TeamUpdateCommand),
    TeamDelete(TeamDeleteCommand),
    ProjectCreate(ProjectCreateCommand),
    ProjectAoJson(ProjectAoJsonCommand),
    ProjectConfigGet(ProjectConfigGetCommand),
    ProjectDiscover(ProjectDiscoverCommand),
    ProjectEvents(ProjectEventsCommand),
    ProjectGet(ProjectGetCommand),
    ProjectHostAssign(ProjectHostAssignCommand),
    ProjectHostClear(ProjectHostClearCommand),
    ProjectHostList(ProjectHostListCommand),
    ProjectList(ProjectListCommand),
    ProjectUpdate(ProjectUpdateCommand),
    ProjectDelete(ProjectDeleteCommand),
    ScheduleCreate(ScheduleCreateCommand),
    ScheduleGet(ScheduleGetCommand),
    ScheduleList(ScheduleListCommand),
    ScheduleUpdate(ScheduleUpdateCommand),
    ScheduleDelete(ScheduleDeleteCommand),
    KnowledgeSourceUpsert(KnowledgeSourceUpsertCommand),
    KnowledgeSourceList(KnowledgeSourceListCommand),
    KnowledgeDocumentCreate(KnowledgeDocumentCreateCommand),
    KnowledgeDocumentList(KnowledgeDocumentListCommand),
    KnowledgeFactCreate(KnowledgeFactCreateCommand),
    KnowledgeFactList(KnowledgeFactListCommand),
    KnowledgeSearch(KnowledgeSearchCommand),
    DaemonOverrideUpsert(DaemonOverrideUpsertCommand),
    DaemonOverrideList(DaemonOverrideListCommand),
    DaemonOverrideClear(DaemonOverrideClearCommand),
    DaemonStatus(DaemonStatusCommand),
    DaemonReconcile(DaemonReconcileCommand),
    McpList(McpListCommand),
    McpServe(McpServeCommand),
}
