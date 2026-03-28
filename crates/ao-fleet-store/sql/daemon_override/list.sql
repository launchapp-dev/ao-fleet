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
ORDER BY updated_at DESC, team_id ASC
