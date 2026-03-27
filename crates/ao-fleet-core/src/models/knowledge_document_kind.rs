use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeDocumentKind {
    Brief,
    Decision,
    Runbook,
    ResearchNote,
    TeamProfile,
    ProjectProfile,
    IncidentReport,
    PolicyNote,
}
