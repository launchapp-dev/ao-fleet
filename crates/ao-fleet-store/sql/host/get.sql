SELECT
  id,
  slug,
  name,
  address,
  platform,
  status,
  capacity_slots,
  created_at,
  updated_at
FROM hosts
WHERE id = ?
