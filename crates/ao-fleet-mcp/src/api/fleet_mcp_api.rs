use ao_fleet_core::{KnowledgeDocument, KnowledgeFact, KnowledgeSource, Project, Schedule, Team};
use ao_fleet_store::{FleetOverview, FleetOverviewQuery};

use crate::error::fleet_mcp_error::FleetMcpError;
use crate::inputs::daemon_reconcile_input::DaemonReconcileInput;
use crate::inputs::knowledge_document_create_input::KnowledgeDocumentCreateInput;
use crate::inputs::knowledge_fact_create_input::KnowledgeFactCreateInput;
use crate::inputs::knowledge_record_list_input::KnowledgeRecordListInput;
use crate::inputs::knowledge_source_upsert_input::KnowledgeSourceUpsertInput;
use crate::inputs::project_create_input::ProjectCreateInput;
use crate::inputs::project_list_input::ProjectListInput;
use crate::inputs::schedule_create_input::ScheduleCreateInput;
use crate::inputs::schedule_list_input::ScheduleListInput;
use crate::inputs::team_create_input::TeamCreateInput;
use crate::inputs::team_list_input::TeamListInput;
use crate::results::daemon_reconcile_result::DaemonReconcileResult;

pub trait FleetMcpApi {
    fn fleet_overview(&self, input: FleetOverviewQuery) -> Result<FleetOverview, FleetMcpError>;

    fn list_teams(&self, input: TeamListInput) -> Result<Vec<Team>, FleetMcpError>;

    fn create_team(&self, input: TeamCreateInput) -> Result<Team, FleetMcpError>;

    fn list_projects(&self, input: ProjectListInput) -> Result<Vec<Project>, FleetMcpError>;

    fn create_project(&self, input: ProjectCreateInput) -> Result<Project, FleetMcpError>;

    fn list_schedules(&self, input: ScheduleListInput) -> Result<Vec<Schedule>, FleetMcpError>;

    fn create_schedule(&self, input: ScheduleCreateInput) -> Result<Schedule, FleetMcpError>;

    fn list_knowledge_sources(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeSource>, FleetMcpError>;

    fn list_knowledge_documents(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeDocument>, FleetMcpError>;

    fn list_knowledge_facts(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeFact>, FleetMcpError>;

    fn upsert_knowledge_source(
        &self,
        input: KnowledgeSourceUpsertInput,
    ) -> Result<KnowledgeSource, FleetMcpError>;

    fn create_knowledge_document(
        &self,
        input: KnowledgeDocumentCreateInput,
    ) -> Result<KnowledgeDocument, FleetMcpError>;

    fn create_knowledge_fact(
        &self,
        input: KnowledgeFactCreateInput,
    ) -> Result<KnowledgeFact, FleetMcpError>;

    fn reconcile_daemons(
        &self,
        input: DaemonReconcileInput,
    ) -> Result<DaemonReconcileResult, FleetMcpError>;
}
