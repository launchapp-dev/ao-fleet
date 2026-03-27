SELECT
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
FROM knowledge_facts
WHERE id = ?1
LIMIT 1;
