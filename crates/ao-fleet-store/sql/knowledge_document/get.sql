SELECT
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
FROM knowledge_documents
WHERE id = ?1
LIMIT 1;
