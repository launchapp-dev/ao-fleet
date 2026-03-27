UPDATE hosts
SET slug = ?,
    name = ?,
    address = ?,
    platform = ?,
    status = ?,
    capacity_slots = ?,
    updated_at = ?
WHERE id = ?
