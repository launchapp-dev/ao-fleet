use serde::{Deserialize, Serialize};

use ao_fleet_core::{KnowledgeDocument, KnowledgeFact};

use crate::KnowledgeQuery;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KnowledgeQueryEngine;

impl KnowledgeQueryEngine {
    pub fn matches_document(&self, query: &KnowledgeQuery, document: &KnowledgeDocument) -> bool {
        self.matches_scope(query, &document.scope)
            && self.matches_document_kind(query, &document.kind)
            && self.matches_source_kind(query, document.source_kind.as_ref())
            && self.matches_text(query, &document.title, &document.summary, &document.body)
            && self.matches_tags(query, &document.tags)
    }

    pub fn matches_fact(&self, query: &KnowledgeQuery, fact: &KnowledgeFact) -> bool {
        self.matches_scope(query, &fact.scope)
            && self.matches_fact_kind(query, &fact.kind)
            && self.matches_source_kind(query, fact.source_kind.as_ref())
            && self.matches_text(query, &fact.statement, "", "")
            && self.matches_tags(query, &fact.tags)
    }

    fn matches_scope(&self, query: &KnowledgeQuery, scope: &ao_fleet_core::KnowledgeScope) -> bool {
        query.scope.as_ref().map_or(true, |expected| expected == scope)
    }

    fn matches_document_kind(
        &self,
        query: &KnowledgeQuery,
        kind: &ao_fleet_core::KnowledgeDocumentKind,
    ) -> bool {
        query.document_kinds.is_empty()
            || query.document_kinds.iter().any(|candidate| candidate == kind)
    }

    fn matches_fact_kind(
        &self,
        query: &KnowledgeQuery,
        kind: &ao_fleet_core::KnowledgeFactKind,
    ) -> bool {
        query.fact_kinds.is_empty() || query.fact_kinds.iter().any(|candidate| candidate == kind)
    }

    fn matches_source_kind(
        &self,
        query: &KnowledgeQuery,
        kind: Option<&ao_fleet_core::KnowledgeSourceKind>,
    ) -> bool {
        query.source_kinds.is_empty()
            || kind
                .is_some_and(|value| query.source_kinds.iter().any(|candidate| candidate == value))
    }

    fn matches_text(&self, query: &KnowledgeQuery, a: &str, b: &str, c: &str) -> bool {
        query.text.as_ref().map_or(true, |needle| {
            let needle = needle.to_ascii_lowercase();
            a.to_ascii_lowercase().contains(&needle)
                || b.to_ascii_lowercase().contains(&needle)
                || c.to_ascii_lowercase().contains(&needle)
        })
    }

    fn matches_tags(&self, query: &KnowledgeQuery, tags: &[String]) -> bool {
        query.tags.iter().all(|tag| tags.iter().any(|candidate| candidate == tag))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use ao_fleet_core::{
        KnowledgeDocument, KnowledgeDocumentKind, KnowledgeFact, KnowledgeFactKind, KnowledgeScope,
        KnowledgeSourceKind,
    };

    use crate::KnowledgeQuery;

    use super::KnowledgeQueryEngine;

    #[test]
    fn matches_document_by_scope_tag_and_text() {
        let engine = KnowledgeQueryEngine;
        let query = KnowledgeQuery {
            scope: Some(KnowledgeScope::Global),
            document_kinds: vec![KnowledgeDocumentKind::Brief],
            fact_kinds: Vec::new(),
            source_kinds: Vec::new(),
            tags: vec!["fleet".to_string()],
            text: Some("knowledge".to_string()),
            limit: 10,
        };
        let document = KnowledgeDocument {
            id: "doc-1".to_string(),
            scope: KnowledgeScope::Global,
            kind: KnowledgeDocumentKind::Brief,
            title: "Fleet knowledge base".to_string(),
            summary: "Shared memory for the company".to_string(),
            body: "This document stores fleet knowledge".to_string(),
            source_id: None,
            source_kind: Some(KnowledgeSourceKind::ManualNote),
            tags: vec!["fleet".to_string(), "memory".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(engine.matches_document(&query, &document));
    }

    #[test]
    fn matches_fact_by_scope_and_text() {
        let engine = KnowledgeQueryEngine;
        let query = KnowledgeQuery {
            scope: Some(KnowledgeScope::Global),
            document_kinds: Vec::new(),
            fact_kinds: vec![KnowledgeFactKind::Policy],
            source_kinds: Vec::new(),
            tags: Vec::new(),
            text: Some("company".to_string()),
            limit: 10,
        };
        let fact = KnowledgeFact {
            id: "fact-1".to_string(),
            scope: KnowledgeScope::Global,
            kind: KnowledgeFactKind::Policy,
            statement: "The company owns the fleet layer".to_string(),
            confidence: 90,
            source_id: Some("source-1".to_string()),
            source_kind: Some(KnowledgeSourceKind::ManualNote),
            tags: vec!["policy".to_string()],
            observed_at: Utc::now(),
            created_at: Utc::now(),
        };

        assert!(engine.matches_fact(&query, &fact));
        assert_eq!(json!({"ok": true}), json!({"ok": true}));
    }
}
