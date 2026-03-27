SELECT id, team_id, timezone, policy_kind, windows_json, enabled, created_at, updated_at
FROM schedules
WHERE team_id = ?
ORDER BY created_at ASC, id ASC
