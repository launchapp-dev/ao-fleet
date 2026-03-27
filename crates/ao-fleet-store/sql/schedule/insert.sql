INSERT INTO schedules (
  id,
  team_id,
  timezone,
  policy_kind,
  windows_json,
  enabled,
  created_at,
  updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
