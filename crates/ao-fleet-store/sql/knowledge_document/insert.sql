INSERT INTO knowledge_documents (
    id,
    scope,
    scope_ref,
    kind,
    title,
    summary,
    body,
    source_id,
    source_kind,
    tags_json,
    created_at,
    updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);
