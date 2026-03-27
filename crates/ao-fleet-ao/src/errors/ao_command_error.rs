use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AoCommandError {
    #[error("failed to invoke ao binary at {binary_path}: {source}")]
    Io {
        binary_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("ao command failed for {binary_path} with exit code {status:?}: {stderr}")]
    CommandFailed { binary_path: PathBuf, status: Option<i32>, stderr: String },
    #[error("invalid utf-8 from {binary_path}: {message}")]
    InvalidUtf8 { binary_path: PathBuf, message: String },
    #[error(
        "invalid json from {binary_path} for project {project_root}: {message}. output={output}"
    )]
    InvalidJson { binary_path: PathBuf, project_root: PathBuf, message: String, output: String },
    #[error(
        "invalid response from {binary_path} for project {project_root}: {message}. output={output}"
    )]
    InvalidResponse { binary_path: PathBuf, project_root: PathBuf, message: String, output: String },
}

impl AoCommandError {
    pub(crate) fn io(binary_path: PathBuf, source: std::io::Error) -> Self {
        Self::Io { binary_path, source }
    }

    pub(crate) fn command_failed(
        binary_path: PathBuf,
        status: Option<i32>,
        stderr: String,
    ) -> Self {
        Self::CommandFailed { binary_path, status, stderr }
    }

    pub(crate) fn invalid_utf8(binary_path: PathBuf, message: String) -> Self {
        Self::InvalidUtf8 { binary_path, message }
    }

    pub(crate) fn invalid_json(
        binary_path: PathBuf,
        project_root: PathBuf,
        message: String,
        output: String,
    ) -> Self {
        Self::InvalidJson { binary_path, project_root, message, output }
    }

    pub(crate) fn invalid_response(
        binary_path: PathBuf,
        project_root: PathBuf,
        message: String,
        output: String,
    ) -> Self {
        Self::InvalidResponse { binary_path, project_root, message, output }
    }
}
