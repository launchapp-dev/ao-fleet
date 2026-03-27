SELECT
  project_id,
  host_id,
  assignment_source,
  assigned_at
FROM project_host_placements
ORDER BY assigned_at DESC, project_id ASC
