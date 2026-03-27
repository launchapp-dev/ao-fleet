use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::config_snapshot_export_command::ConfigSnapshotExportCommand;
use crate::cli::handlers::fleet_config_snapshot::FleetConfigSnapshot;
use crate::cli::handlers::json_printer::print_json;

pub fn config_snapshot_export(db_path: &str, command: ConfigSnapshotExportCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let snapshot = FleetConfigSnapshot {
        version: "v1".to_string(),
        exported_at: chrono::Utc::now(),
        teams: store.list_teams()?,
        projects: store.list_projects(None)?,
        schedules: store.list_schedules(None)?,
    };

    match command.output {
        Some(path) => {
            std::fs::write(path, serde_json::to_string_pretty(&snapshot)?)?;
            Ok(())
        }
        None => print_json(&snapshot),
    }
}
