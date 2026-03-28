SELECT
  id,
  team_id,
  mode,
  forced_state,
  pause_until,
  note,
  source,
  created_at,
  updated_at
FROM daemon_overrides
WHERE team_id = ?
LIMIT 1
