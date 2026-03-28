CREATE TABLE IF NOT EXISTS daemon_overrides (
  id TEXT PRIMARY KEY,
  team_id TEXT NOT NULL UNIQUE REFERENCES teams(id) ON DELETE CASCADE,
  mode TEXT NOT NULL,
  forced_state TEXT,
  pause_until TEXT,
  note TEXT,
  source TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_daemon_overrides_team_id
  ON daemon_overrides(team_id);
