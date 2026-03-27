use chrono::Utc;
use serde::{Deserialize, Serialize};

use ao_fleet_core::KnowledgeSource;

use crate::KnowledgeIngestionJob;
use crate::KnowledgeIngestionJobStatus;

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
}

#[cfg(test)]
mod tests {
    use ao_fleet_core::{KnowledgeScope, KnowledgeSource, KnowledgeSourceKind, KnowledgeSyncState};
    use chrono::Utc;
    use serde_json::json;

    use super::KnowledgeIngestionService;

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
}
