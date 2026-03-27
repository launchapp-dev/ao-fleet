use serde::{Deserialize, Serialize};

use crate::KnowledgeCollection;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct KnowledgeCatalog {
    pub collections: Vec<KnowledgeCollection>,
}

impl KnowledgeCatalog {
    pub fn list_collections(&self) -> &[KnowledgeCollection] {
        &self.collections
    }
}
