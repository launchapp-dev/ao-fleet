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
WHERE (?1 IS NULL OR scope = ?1)
  AND (?2 IS NULL OR scope_ref = ?2)
ORDER BY updated_at DESC
LIMIT ?3;
