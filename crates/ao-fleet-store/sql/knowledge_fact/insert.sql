INSERT INTO knowledge_facts (
    id,
    scope,
    scope_ref,
    kind,
    statement,
    confidence,
    source_id,
    source_kind,
    tags_json,
    observed_at,
    created_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11);
