INSERT INTO audit_events (
    id,
    team_id,
    entity_type,
    entity_id,
    action,
    actor_type,
    actor_id,
    summary,
    details_json,
    occurred_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
