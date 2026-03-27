SELECT
  project_id,
  team_id,
  observed_state,
  source,
  checked_at,
  details_json
FROM observed_daemon_statuses
WHERE (?1 IS NULL OR team_id = ?1)
ORDER BY checked_at DESC, project_id ASC
