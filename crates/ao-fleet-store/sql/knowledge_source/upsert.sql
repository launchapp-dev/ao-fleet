INSERT INTO knowledge_sources (
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
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT(id) DO UPDATE SET
    kind = excluded.kind,
    label = excluded.label,
    uri = excluded.uri,
    scope = excluded.scope,
    scope_ref = excluded.scope_ref,
    sync_state = excluded.sync_state,
    last_synced_at = excluded.last_synced_at,
    metadata_json = excluded.metadata_json,
    created_at = excluded.created_at,
    updated_at = excluded.updated_at;
