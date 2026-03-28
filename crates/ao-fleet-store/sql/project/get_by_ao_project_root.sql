SELECT id,
       team_id,
       slug,
       root_path,
       ao_project_root,
       default_branch,
       remote_url,
       enabled,
       created_at,
       updated_at
FROM projects
WHERE ao_project_root = ?1
LIMIT 1;
