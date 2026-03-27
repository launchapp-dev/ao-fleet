SELECT id, team_id, timezone, policy_kind, windows_json, enabled, created_at, updated_at
FROM schedules
ORDER BY created_at ASC, id ASC
