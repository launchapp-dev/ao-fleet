CREATE TABLE IF NOT EXISTS hosts (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  address TEXT NOT NULL UNIQUE,
  platform TEXT NOT NULL,
  status TEXT NOT NULL,
  capacity_slots INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_host_placements (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
  assignment_source TEXT NOT NULL,
  assigned_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_project_host_placements_host_id
  ON project_host_placements(host_id);
