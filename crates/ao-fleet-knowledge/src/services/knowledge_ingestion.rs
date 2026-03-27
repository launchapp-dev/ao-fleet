use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::KnowledgeBaseManifest;
use crate::KnowledgeIngestionJob;
use crate::KnowledgeIngestionJobStatus;
use crate::KnowledgeSource;
use crate::KnowledgeSourceKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KnowledgeIngestionService;

impl KnowledgeIngestionService {
    pub fn plan_job(
        &self,
        manifest_id: impl Into<String>,
        source: &KnowledgeSource,
        collection_id: impl Into<String>,
    ) -> KnowledgeIngestionJob {
        let requested_at = Utc::now();

        KnowledgeIngestionJob {
            id: format!("job-{}", uuid::Uuid::now_v7()),
            manifest_id: manifest_id.into(),
            source_id: source.id.clone(),
            collection_id: collection_id.into(),
            status: KnowledgeIngestionJobStatus::Pending,
            requested_at,
            started_at: None,
            finished_at: None,
        }
    }

    pub fn plan_automatic_jobs(
        &self,
        manifest: &KnowledgeBaseManifest,
        sources: &[KnowledgeSource],
        collection_id: impl Into<String>,
    ) -> Vec<KnowledgeIngestionJob> {
        let collection_id = collection_id.into();

        sources
            .iter()
            .filter(|source| manifest.source_kinds.contains(&source.kind))
            .filter(|source| is_automatic_source_kind(&source.kind))
            .map(|source| self.plan_job(manifest.id.clone(), source, collection_id.clone()))
            .collect()
    }
}

fn is_automatic_source_kind(kind: &KnowledgeSourceKind) -> bool {
    !matches!(kind, KnowledgeSourceKind::ManualNote)
}

#[cfg(test)]
mod tests {
    use ao_fleet_core::{
        KnowledgeDocumentKind, KnowledgeScope, KnowledgeSource, KnowledgeSourceKind,
        KnowledgeSyncState,
    };
    use chrono::Utc;
    use serde_json::json;

    use super::KnowledgeIngestionService;
    use crate::KnowledgeBaseManifest;

    #[test]
    fn plans_pending_jobs_from_sources() {
        let service = KnowledgeIngestionService;
        let source = KnowledgeSource {
            id: "source-1".to_string(),
            kind: KnowledgeSourceKind::ManualNote,
            label: "Operator note".to_string(),
            uri: None,
            scope: KnowledgeScope::Global,
            scope_ref: None,
            sync_state: KnowledgeSyncState::Pending,
            last_synced_at: None,
            metadata: json!({"note": "seed"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let job = service.plan_job("manifest-1", &source, "collection-1");

        assert_eq!(job.manifest_id, "manifest-1");
        assert_eq!(job.source_id, "source-1");
        assert_eq!(job.collection_id, "collection-1");
    }

    #[test]
    fn plans_automatic_jobs_for_non_manual_sources() {
        let service = KnowledgeIngestionService;
        let manifest = KnowledgeBaseManifest {
            id: "manifest-1".to_string(),
            company_id: "company-1".to_string(),
            name: "Company Knowledge Base".to_string(),
            description: "Company memory".to_string(),
            root_path: "/tmp/ao-fleet".to_string(),
            retention_days: 365,
            embedding_enabled: false,
            scopes: vec![KnowledgeScope::Global],
            source_kinds: vec![
                KnowledgeSourceKind::ManualNote,
                KnowledgeSourceKind::GitHubIssue,
                KnowledgeSourceKind::AoEvent,
            ],
            document_kinds: vec![KnowledgeDocumentKind::Brief],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let sources = vec![
            KnowledgeSource {
                id: "manual-1".to_string(),
                kind: KnowledgeSourceKind::ManualNote,
                label: "Operator note".to_string(),
                uri: None,
                scope: KnowledgeScope::Global,
                scope_ref: None,
                sync_state: KnowledgeSyncState::Pending,
                last_synced_at: None,
                metadata: json!({"source": "manual"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            KnowledgeSource {
                id: "issue-1".to_string(),
                kind: KnowledgeSourceKind::GitHubIssue,
                label: "Bug report".to_string(),
                uri: Some("https://github.com/acme/app/issues/1".to_string()),
                scope: KnowledgeScope::Project,
                scope_ref: Some("project-1".to_string()),
                sync_state: KnowledgeSyncState::Ready,
                last_synced_at: Some(Utc::now()),
                metadata: json!({"source": "github"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            KnowledgeSource {
                id: "event-1".to_string(),
                kind: KnowledgeSourceKind::AoEvent,
                label: "Daemon event".to_string(),
                uri: None,
                scope: KnowledgeScope::Operational,
                scope_ref: None,
                sync_state: KnowledgeSyncState::Stale,
                last_synced_at: None,
                metadata: json!({"source": "ao"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let jobs = service.plan_automatic_jobs(&manifest, &sources, "collection-1");

        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].source_id, "issue-1");
        assert_eq!(jobs[1].source_id, "event-1");
        assert!(jobs.iter().all(|job| job.manifest_id == "manifest-1"));
        assert!(jobs.iter().all(|job| job.collection_id == "collection-1"));
    }
}
