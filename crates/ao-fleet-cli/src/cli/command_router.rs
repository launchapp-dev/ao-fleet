use anyhow::Result;

use crate::cli::handlers::audit_list::audit_list;
use crate::cli::handlers::config_snapshot_export::config_snapshot_export;
use crate::cli::handlers::config_snapshot_import::config_snapshot_import;
use crate::cli::handlers::daemon_override_clear::daemon_override_clear;
use crate::cli::handlers::daemon_override_list::daemon_override_list;
use crate::cli::handlers::daemon_override_upsert::daemon_override_upsert;
use crate::cli::handlers::daemon_reconcile::daemon_reconcile;
use crate::cli::handlers::daemon_status::daemon_status;
use crate::cli::handlers::db_init::db_init;
use crate::cli::handlers::fleet_overview::fleet_overview;
use crate::cli::handlers::host_create::host_create;
use crate::cli::handlers::host_delete::host_delete;
use crate::cli::handlers::host_get::host_get;
use crate::cli::handlers::host_list::host_list;
use crate::cli::handlers::host_update::host_update;
use crate::cli::handlers::knowledge_document_create::knowledge_document_create;
use crate::cli::handlers::knowledge_document_list::knowledge_document_list;
use crate::cli::handlers::knowledge_fact_create::knowledge_fact_create;
use crate::cli::handlers::knowledge_fact_list::knowledge_fact_list;
use crate::cli::handlers::knowledge_search::knowledge_search;
use crate::cli::handlers::knowledge_source_list::knowledge_source_list;
use crate::cli::handlers::knowledge_source_upsert::knowledge_source_upsert;
use crate::cli::handlers::mcp_list::mcp_list;
use crate::cli::handlers::mcp_serve::mcp_serve;
use crate::cli::handlers::project_ao_json::project_ao_json;
use crate::cli::handlers::project_config_get::project_config_get;
use crate::cli::handlers::project_create::project_create;
use crate::cli::handlers::project_delete::project_delete;
use crate::cli::handlers::project_discover::project_discover;
use crate::cli::handlers::project_events::project_events;
use crate::cli::handlers::project_get::project_get;
use crate::cli::handlers::project_host_assign::project_host_assign;
use crate::cli::handlers::project_host_clear::project_host_clear;
use crate::cli::handlers::project_host_list::project_host_list;
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
        CommandGroup::ConfigSnapshotExport(command) => {
            config_snapshot_export(&root.db_path, command)
        }
        CommandGroup::ConfigSnapshotImport(command) => {
            config_snapshot_import(&root.db_path, command)
        }
        CommandGroup::FleetOverview(command) => fleet_overview(&root.db_path, command),
        CommandGroup::HostCreate(command) => host_create(&root.db_path, command),
        CommandGroup::HostGet(command) => host_get(&root.db_path, command),
        CommandGroup::HostList(command) => host_list(&root.db_path, command),
        CommandGroup::HostUpdate(command) => host_update(&root.db_path, command),
        CommandGroup::HostDelete(command) => host_delete(&root.db_path, command),
        CommandGroup::TeamCreate(command) => team_create(&root.db_path, command),
        CommandGroup::TeamGet(command) => team_get(&root.db_path, command),
        CommandGroup::TeamList(command) => team_list(&root.db_path, command),
        CommandGroup::TeamUpdate(command) => team_update(&root.db_path, command),
        CommandGroup::TeamDelete(command) => team_delete(&root.db_path, command),
        CommandGroup::ProjectCreate(command) => project_create(&root.db_path, command),
        CommandGroup::ProjectAoJson(command) => project_ao_json(&root.db_path, command),
        CommandGroup::ProjectConfigGet(command) => project_config_get(&root.db_path, command),
        CommandGroup::ProjectDiscover(command) => project_discover(&root.db_path, command),
        CommandGroup::ProjectEvents(command) => project_events(&root.db_path, command),
        CommandGroup::ProjectGet(command) => project_get(&root.db_path, command),
        CommandGroup::ProjectHostAssign(command) => project_host_assign(&root.db_path, command),
        CommandGroup::ProjectHostClear(command) => project_host_clear(&root.db_path, command),
        CommandGroup::ProjectHostList(command) => project_host_list(&root.db_path, command),
        CommandGroup::ProjectList(command) => project_list(&root.db_path, command),
        CommandGroup::ProjectUpdate(command) => project_update(&root.db_path, command),
        CommandGroup::ProjectDelete(command) => project_delete(&root.db_path, command),
        CommandGroup::ScheduleCreate(command) => schedule_create(&root.db_path, command),
        CommandGroup::ScheduleGet(command) => schedule_get(&root.db_path, command),
        CommandGroup::ScheduleList(command) => schedule_list(&root.db_path, command),
        CommandGroup::ScheduleUpdate(command) => schedule_update(&root.db_path, command),
        CommandGroup::ScheduleDelete(command) => schedule_delete(&root.db_path, command),
        CommandGroup::KnowledgeSourceUpsert(command) => {
            knowledge_source_upsert(&root.db_path, command)
        }
        CommandGroup::KnowledgeSourceList(command) => knowledge_source_list(&root.db_path, command),
        CommandGroup::KnowledgeDocumentCreate(command) => {
            knowledge_document_create(&root.db_path, command)
        }
        CommandGroup::KnowledgeDocumentList(command) => {
            knowledge_document_list(&root.db_path, command)
        }
        CommandGroup::KnowledgeFactCreate(command) => knowledge_fact_create(&root.db_path, command),
        CommandGroup::KnowledgeFactList(command) => knowledge_fact_list(&root.db_path, command),
        CommandGroup::KnowledgeSearch(command) => knowledge_search(&root.db_path, command),
        CommandGroup::DaemonOverrideUpsert(command) => {
            daemon_override_upsert(&root.db_path, command)
        }
        CommandGroup::DaemonOverrideList(command) => daemon_override_list(&root.db_path, command),
        CommandGroup::DaemonOverrideClear(command) => daemon_override_clear(&root.db_path, command),
        CommandGroup::DaemonStatus(command) => daemon_status(&root.db_path, command),
        CommandGroup::DaemonReconcile(command) => daemon_reconcile(&root.db_path, command),
        CommandGroup::McpList(command) => mcp_list(command),
        CommandGroup::McpServe(command) => mcp_serve(&root.db_path, command),
    }
}
