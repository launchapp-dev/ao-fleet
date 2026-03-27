UPDATE schedules
SET team_id = ?,
    timezone = ?,
    policy_kind = ?,
    windows_json = ?,
    enabled = ?,
    updated_at = ?
WHERE id = ?
