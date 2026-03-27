use ao_fleet_core::FleetError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("validation failed: {message}")]
    Validation { message: String },
    #[error("not found: {entity} {id}")]
    NotFound { entity: &'static str, id: String },
    #[error("already exists: {entity} {key}")]
    AlreadyExists { entity: &'static str, key: String },
}

impl StoreError {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }

    pub(crate) fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound { entity, id: id.into() }
    }

    pub(crate) fn already_exists(entity: &'static str, key: impl Into<String>) -> Self {
        Self::AlreadyExists { entity, key: key.into() }
    }
}

impl From<FleetError> for StoreError {
    fn from(value: FleetError) -> Self {
        match value {
            FleetError::Validation { message } => Self::validation(message),
        }
    }
}
