use serde::{Deserialize, Serialize};

use crate::{
    KnowledgeDocument, KnowledgeFact, KnowledgeQuery, KnowledgeQueryEngine, KnowledgeSearchResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KnowledgeSearchService {
    engine: KnowledgeQueryEngine,
}

impl KnowledgeSearchService {
    pub fn search(
        &self,
        query: &KnowledgeQuery,
        documents: &[KnowledgeDocument],
        facts: &[KnowledgeFact],
    ) -> KnowledgeSearchResult {
        let documents = documents
            .iter()
            .filter(|document| self.engine.matches_document(query, document))
            .take(query.limit)
            .cloned()
            .collect();

        let facts = facts
            .iter()
            .filter(|fact| self.engine.matches_fact(query, fact))
            .take(query.limit)
            .cloned()
            .collect();

        KnowledgeSearchResult { documents, facts }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use ao_fleet_core::{
        KnowledgeDocument, KnowledgeDocumentKind, KnowledgeFact, KnowledgeFactKind, KnowledgeScope,
    };

    use crate::{KnowledgeQuery, KnowledgeSearchService};

    #[test]
    fn search_filters_documents_and_facts() {
        let service = KnowledgeSearchService::default();
        let query = KnowledgeQuery {
            scope: Some(KnowledgeScope::Team),
            scope_ref: Some("team-1".to_string()),
            document_kinds: vec![KnowledgeDocumentKind::Runbook],
            fact_kinds: vec![KnowledgeFactKind::Policy],
            source_kinds: Vec::new(),
            tags: vec!["ops".to_string()],
            text: Some("restart".to_string()),
            limit: 10,
        };
        let documents = vec![KnowledgeDocument {
            id: "document-1".to_string(),
            scope: KnowledgeScope::Team,
            scope_ref: Some("team-1".to_string()),
            kind: KnowledgeDocumentKind::Runbook,
            title: "Restart".to_string(),
            summary: "Restart steps".to_string(),
            body: "Restart the daemon".to_string(),
            source_id: None,
            source_kind: None,
            tags: vec!["ops".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let facts = vec![KnowledgeFact {
            id: "fact-1".to_string(),
            scope: KnowledgeScope::Team,
            scope_ref: Some("team-1".to_string()),
            kind: KnowledgeFactKind::Policy,
            statement: "restart policy".to_string(),
            confidence: 90,
            source_id: None,
            source_kind: None,
            tags: vec!["ops".to_string()],
            observed_at: Utc::now(),
            created_at: Utc::now(),
        }];

        let result = service.search(&query, &documents, &facts);

        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.facts.len(), 1);
    }
}
