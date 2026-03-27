UPDATE projects
SET team_id = ?,
    slug = ?,
    root_path = ?,
    ao_project_root = ?,
    default_branch = ?,
    remote_url = ?,
    enabled = ?,
    updated_at = ?
WHERE id = ?
