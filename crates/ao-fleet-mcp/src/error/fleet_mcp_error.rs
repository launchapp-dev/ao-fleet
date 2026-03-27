use thiserror::Error;

#[derive(Debug, Error)]
pub enum FleetMcpError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Store(#[from] ao_fleet_store::StoreError),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    ParseTime(#[from] chrono::ParseError),
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl FleetMcpError {
    pub fn code(&self) -> i64 {
        match self {
            Self::InvalidRequest(_) => -32600,
            Self::UnknownTool(_) => -32601,
            Self::Validation(_) => -32602,
            Self::Store(_) | Self::Json(_) | Self::Io(_) | Self::ParseTime(_) => -32603,
        }
    }
}
