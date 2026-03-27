use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeError {
    #[error("validation failed: {message}")]
    Validation { message: String },
}
