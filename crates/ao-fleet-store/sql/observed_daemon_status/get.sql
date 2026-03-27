SELECT
  project_id,
  team_id,
  observed_state,
  source,
  checked_at,
  details_json
FROM observed_daemon_statuses
WHERE project_id = ?1
