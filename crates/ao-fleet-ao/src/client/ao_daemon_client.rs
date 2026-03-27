use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ao_command::{
    build_daemon_pause_args, build_daemon_resume_args, build_daemon_start_args,
    build_daemon_status_args, build_daemon_stop_args, build_project_status_args,
};
use crate::errors::ao_command_error::AoCommandError;
use crate::models::daemon_command_result::DaemonCommandResult;
use crate::models::daemon_start_options::DaemonStartOptions;
use crate::models::daemon_state::DaemonState;
use crate::models::project_status_report::ProjectStatusReport;

#[derive(Debug, Clone)]
pub struct AoDaemonClient {
    binary_path: PathBuf,
}

impl AoDaemonClient {
    pub fn new() -> Self {
        Self { binary_path: resolve_default_binary_path() }
    }

    pub fn with_binary_path(binary_path: impl Into<PathBuf>) -> Self {
        Self { binary_path: binary_path.into() }
    }

    pub fn daemon_status(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<DaemonState, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self.run_json(project_root, build_daemon_status_args(project_root))?;
        parse_daemon_state(project_root, value)
    }

    pub fn project_status(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<ProjectStatusReport, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self.run_json(project_root, build_project_status_args(project_root))?;
        ProjectStatusReport::from_cli_value(project_root, value)
    }

    pub fn start(
        &self,
        project_root: impl AsRef<Path>,
        options: &DaemonStartOptions,
    ) -> Result<DaemonCommandResult, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self.run_json(project_root, build_daemon_start_args(project_root, options))?;
        Ok(DaemonCommandResult::from_cli_value("start", value))
    }

    pub fn stop(
        &self,
        project_root: impl AsRef<Path>,
        shutdown_timeout_secs: Option<u64>,
    ) -> Result<DaemonCommandResult, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self
            .run_json(project_root, build_daemon_stop_args(project_root, shutdown_timeout_secs))?;
        Ok(DaemonCommandResult::from_cli_value("stop", value))
    }

    pub fn pause(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<DaemonCommandResult, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self.run_json(project_root, build_daemon_pause_args(project_root))?;
        Ok(DaemonCommandResult::from_cli_value("pause", value))
    }

    pub fn resume(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<DaemonCommandResult, AoCommandError> {
        let project_root = project_root.as_ref();
        let value = self.run_json(project_root, build_daemon_resume_args(project_root))?;
        Ok(DaemonCommandResult::from_cli_value("resume", value))
    }

    fn run_json(
        &self,
        project_root: &Path,
        args: Vec<std::ffi::OsString>,
    ) -> Result<serde_json::Value, AoCommandError> {
        let output = Command::new(&self.binary_path)
            .args(args)
            .output()
            .map_err(|source| AoCommandError::io(self.binary_path.clone(), source))?;

        if !output.status.success() {
            return Err(AoCommandError::command_failed(
                self.binary_path.clone(),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|error| {
            AoCommandError::invalid_utf8(self.binary_path.clone(), error.to_string())
        })?;
        parse_envelope(self.binary_path.as_path(), project_root, &stdout)
    }
}

fn resolve_default_binary_path() -> PathBuf {
    if let Some(explicit) = std::env::var_os("AO_BIN") {
        return PathBuf::from(explicit);
    }

    if let Some(home) = std::env::var_os("HOME") {
        let local_bin = PathBuf::from(home).join(".local/bin/ao");
        if local_bin.exists() {
            return local_bin;
        }
    }

    PathBuf::from("ao")
}

fn parse_envelope(
    binary_path: &Path,
    project_root: &Path,
    stdout: &str,
) -> Result<serde_json::Value, AoCommandError> {
    let envelope: AoCliEnvelope<serde_json::Value> =
        serde_json::from_str(stdout).map_err(|error| {
            AoCommandError::invalid_json(
                binary_path.to_path_buf(),
                project_root.to_path_buf(),
                error.to_string(),
                stdout.to_string(),
            )
        })?;

    if !envelope.ok {
        return Err(AoCommandError::invalid_response(
            binary_path.to_path_buf(),
            project_root.to_path_buf(),
            "CLI envelope reported ok=false".to_string(),
            stdout.to_string(),
        ));
    }

    Ok(envelope.data)
}

fn parse_daemon_state(
    project_root: &Path,
    value: serde_json::Value,
) -> Result<DaemonState, AoCommandError> {
    let state = value.as_str().ok_or_else(|| {
        AoCommandError::invalid_response(
            PathBuf::from("ao"),
            project_root.to_path_buf(),
            "daemon status response was not a string".to_string(),
            value.to_string(),
        )
    })?;

    Ok(DaemonState::from_cli_value(state))
}

#[derive(Debug, serde::Deserialize)]
struct AoCliEnvelope<T> {
    ok: bool,
    data: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_status_parses_string_state() {
        let state = parse_daemon_state(
            Path::new("/tmp/project"),
            serde_json::Value::String("crashed".to_string()),
        )
        .expect("state parses");

        assert_eq!(state, DaemonState::Crashed);
    }
}
