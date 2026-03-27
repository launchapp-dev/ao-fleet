SELECT
  teams.id,
  teams.slug,
  projects.id,
  projects.slug,
  projects.ao_project_root,
  CASE
    WHEN projects.enabled = 1 THEN 'running'
    ELSE 'stopped'
  END AS desired_state,
  observed_daemon_statuses.observed_state,
  observed_daemon_statuses.checked_at,
  observed_daemon_statuses.source,
  observed_daemon_statuses.details_json
FROM projects
INNER JOIN teams ON teams.id = projects.team_id
LEFT JOIN observed_daemon_statuses ON observed_daemon_statuses.project_id = projects.id
WHERE (?1 IS NULL OR teams.id = ?1)
ORDER BY teams.slug ASC, projects.slug ASC
