CREATE TABLE IF NOT EXISTS observed_daemon_statuses (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
  observed_state TEXT NOT NULL,
  source TEXT NOT NULL,
  checked_at TEXT NOT NULL,
  details_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_observed_daemon_statuses_team_id
  ON observed_daemon_statuses(team_id);
