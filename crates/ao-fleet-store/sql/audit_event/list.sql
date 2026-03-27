SELECT
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
FROM audit_events
WHERE (?1 IS NULL OR team_id = ?1)
ORDER BY occurred_at DESC, id DESC
LIMIT ?2;
