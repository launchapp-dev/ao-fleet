use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ao_fleet_core::{KnowledgeDocumentKind, KnowledgeScope, KnowledgeSourceKind};

use crate::KnowledgeError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeBaseManifest {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub description: String,
    pub root_path: String,
    pub retention_days: u16,
    pub embedding_enabled: bool,
    pub scopes: Vec<KnowledgeScope>,
    pub source_kinds: Vec<KnowledgeSourceKind>,
    pub document_kinds: Vec<KnowledgeDocumentKind>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KnowledgeBaseManifest {
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if self.id.trim().is_empty()
            || self.company_id.trim().is_empty()
            || self.name.trim().is_empty()
            || self.root_path.trim().is_empty()
        {
            return Err(KnowledgeError::Validation {
                message: "knowledge base manifest fields cannot be empty".to_string(),
            });
        }

        if self.retention_days == 0 {
            return Err(KnowledgeError::Validation {
                message: "retention_days must be greater than zero".to_string(),
            });
        }

        Ok(())
    }

    pub fn default_for(company_id: impl Into<String>, root_path: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: format!("kb-{}", uuid::Uuid::now_v7()),
            company_id: company_id.into(),
            name: "Company Knowledge Base".to_string(),
            description: "Shared company memory for teams, workflows, and decisions".to_string(),
            root_path: root_path.into(),
            retention_days: 365,
            embedding_enabled: false,
            scopes: vec![KnowledgeScope::Global],
            source_kinds: vec![
                KnowledgeSourceKind::AoEvent,
                KnowledgeSourceKind::GitCommit,
                KnowledgeSourceKind::GitHubIssue,
                KnowledgeSourceKind::GitHubPullRequest,
                KnowledgeSourceKind::ManualNote,
                KnowledgeSourceKind::Incident,
                KnowledgeSourceKind::ScheduleChange,
                KnowledgeSourceKind::WorkflowRun,
            ],
            document_kinds: vec![
                KnowledgeDocumentKind::Brief,
                KnowledgeDocumentKind::Decision,
                KnowledgeDocumentKind::Runbook,
                KnowledgeDocumentKind::ResearchNote,
                KnowledgeDocumentKind::TeamProfile,
                KnowledgeDocumentKind::ProjectProfile,
                KnowledgeDocumentKind::IncidentReport,
                KnowledgeDocumentKind::PolicyNote,
            ],
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KnowledgeBaseManifest;

    #[test]
    fn default_manifest_is_valid() {
        let manifest = KnowledgeBaseManifest::default_for("company-1", "/tmp/ao-fleet");

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.retention_days, 365);
        assert!(!manifest.embedding_enabled);
    }
}
