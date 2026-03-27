SELECT id, slug, name, mission, ownership, business_priority, created_at, updated_at
FROM teams
ORDER BY business_priority DESC, created_at ASC, slug ASC
