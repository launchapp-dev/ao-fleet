CREATE TABLE IF NOT EXISTS knowledge_sources (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    uri TEXT,
    scope TEXT NOT NULL,
    scope_ref TEXT,
    sync_state TEXT NOT NULL,
    last_synced_at TEXT,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_knowledge_sources_scope
    ON knowledge_sources (scope, scope_ref, updated_at DESC);

CREATE TABLE IF NOT EXISTS knowledge_documents (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    scope_ref TEXT,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    body TEXT NOT NULL,
    source_id TEXT,
    source_kind TEXT,
    tags_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES knowledge_sources (id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_knowledge_documents_scope
    ON knowledge_documents (scope, scope_ref, updated_at DESC);

CREATE TABLE IF NOT EXISTS knowledge_facts (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    scope_ref TEXT,
    kind TEXT NOT NULL,
    statement TEXT NOT NULL,
    confidence INTEGER NOT NULL,
    source_id TEXT,
    source_kind TEXT,
    tags_json TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES knowledge_sources (id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_knowledge_facts_scope
    ON knowledge_facts (scope, scope_ref, observed_at DESC);
