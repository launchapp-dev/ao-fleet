use ao_fleet_core::{KnowledgeDocument, KnowledgeFact};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSearchResult {
    pub documents: Vec<KnowledgeDocument>,
    pub facts: Vec<KnowledgeFact>,
}
