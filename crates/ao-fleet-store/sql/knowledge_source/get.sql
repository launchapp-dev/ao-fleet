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
WHERE id = ?1
LIMIT 1;
