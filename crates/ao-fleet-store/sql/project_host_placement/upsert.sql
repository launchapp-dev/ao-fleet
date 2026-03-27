INSERT INTO project_host_placements (
  project_id,
  host_id,
  assignment_source,
  assigned_at
) VALUES (?, ?, ?, ?)
ON CONFLICT(project_id) DO UPDATE SET
  host_id = excluded.host_id,
  assignment_source = excluded.assignment_source,
  assigned_at = excluded.assigned_at
