use std::path::Path;

use anyhow::{Result, bail};

use ao_fleet_mcp::{FleetMcpServer, FleetMcpStoreApi};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::mcp_serve_command::McpServeCommand;

pub fn mcp_serve(db_path: &str, _command: McpServeCommand) -> Result<()> {
    validate_db_path(db_path)?;

    let store = FleetStore::open(db_path)?;
    let api = FleetMcpStoreApi::new(store);
    FleetMcpServer::new(api).serve_stdio()?;
    Ok(())
}

fn validate_db_path(db_path: &str) -> Result<()> {
    let trimmed = db_path.trim();
    if trimmed.is_empty() {
        bail!("db path must not be empty");
    }

    let path = Path::new(trimmed);
    if path.exists() && path.is_dir() {
        bail!("db path must point to a file, not a directory");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::validate_db_path;

    #[test]
    fn validate_db_path_rejects_empty_values() {
        let error = validate_db_path("   ").expect_err("expected validation failure");
        assert!(error.to_string().contains("must not be empty"));
    }

    #[test]
    fn validate_db_path_rejects_directories() {
        let directory = std::env::temp_dir()
            .join(format!("ao-fleet-cli-mcp-serve-test-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("create temp dir");

        let error = validate_db_path(directory.to_str().expect("utf-8 path"))
            .expect_err("expected validation failure");

        fs::remove_dir_all(&directory).expect("clean up temp dir");
        assert!(error.to_string().contains("must point to a file"));
    }
}
