use anyhow::Result;

use ao_fleet_store::FleetStore;

use crate::cli::handlers::db_init_command::DbInitCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn db_init(db_path: &str, _command: DbInitCommand) -> Result<()> {
    FleetStore::open(db_path)?;
    print_json(&serde_json::json!({
        "db_path": db_path,
        "status": "initialized"
    }))
}
