use serde::{Deserialize, Serialize};

use crate::KnowledgeBaseManifest;
use crate::KnowledgeCatalog;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub manifest: KnowledgeBaseManifest,
    pub catalog: KnowledgeCatalog,
}

impl KnowledgeBase {
    pub fn new(manifest: KnowledgeBaseManifest, catalog: KnowledgeCatalog) -> Self {
        Self { manifest, catalog }
    }
}
