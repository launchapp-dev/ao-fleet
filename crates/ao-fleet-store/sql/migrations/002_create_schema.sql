CREATE TABLE IF NOT EXISTS teams (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  mission TEXT NOT NULL,
  ownership TEXT NOT NULL,
  business_priority INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  slug TEXT NOT NULL UNIQUE,
  root_path TEXT NOT NULL UNIQUE,
  ao_project_root TEXT NOT NULL UNIQUE,
  default_branch TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_projects_team_id ON projects(team_id);

CREATE TABLE IF NOT EXISTS schedules (
  id TEXT PRIMARY KEY,
  team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  timezone TEXT NOT NULL,
  policy_kind TEXT NOT NULL,
  windows_json TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_schedules_team_id ON schedules(team_id);
