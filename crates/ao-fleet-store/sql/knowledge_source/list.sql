SELECT
    id,
    kind,
    label,
    uri,
    scope,
    scope_ref,
    sync_state,
    last_synced_at,
    metadata_json,
    created_at,
    updated_at
FROM knowledge_sources
WHERE (?1 IS NULL OR scope = ?1)
  AND (?2 IS NULL OR scope_ref = ?2)
ORDER BY updated_at DESC
LIMIT ?3;
