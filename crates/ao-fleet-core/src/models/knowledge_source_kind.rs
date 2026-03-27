use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceKind {
    AoEvent,
    GitCommit,
    GitHubIssue,
    GitHubPullRequest,
    ManualNote,
    Incident,
    ScheduleChange,
    WorkflowRun,
}
