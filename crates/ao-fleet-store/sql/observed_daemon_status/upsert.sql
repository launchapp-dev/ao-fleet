INSERT INTO observed_daemon_statuses (
  project_id,
  team_id,
  observed_state,
  source,
  checked_at,
  details_json
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(project_id) DO UPDATE SET
  team_id = excluded.team_id,
  observed_state = excluded.observed_state,
  source = excluded.source,
  checked_at = excluded.checked_at,
  details_json = excluded.details_json
