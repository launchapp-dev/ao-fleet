use thiserror::Error;

#[derive(Debug, Error)]
pub enum FleetError {
    #[error("validation failed: {message}")]
    Validation { message: String },
}
