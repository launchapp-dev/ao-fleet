SELECT id, team_id, slug, root_path, ao_project_root, default_branch, enabled, created_at, updated_at
FROM projects
WHERE team_id = ?
ORDER BY created_at ASC, slug ASC
