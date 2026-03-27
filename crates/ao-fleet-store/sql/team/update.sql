UPDATE teams
SET slug = ?,
    name = ?,
    mission = ?,
    ownership = ?,
    business_priority = ?,
    updated_at = ?
WHERE id = ?
