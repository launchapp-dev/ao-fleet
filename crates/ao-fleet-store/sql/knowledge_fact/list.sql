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
WHERE (?1 IS NULL OR scope = ?1)
  AND (?2 IS NULL OR scope_ref = ?2)
ORDER BY observed_at DESC
LIMIT ?3;
