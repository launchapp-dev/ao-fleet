INSERT INTO daemon_overrides (
  id,
  team_id,
  mode,
  forced_state,
  pause_until,
  note,
  source,
  created_at,
  updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
ON CONFLICT(team_id) DO UPDATE SET
  mode = excluded.mode,
  forced_state = excluded.forced_state,
  pause_until = excluded.pause_until,
  note = excluded.note,
  source = excluded.source,
  created_at = excluded.created_at,
  updated_at = excluded.updated_at;
