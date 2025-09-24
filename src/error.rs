/* src/error.rs */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathmapError {
    #[error("Namespace '{0}' already exists")]
    NamespaceAlreadyExists(String),

    #[error("Namespace '{0}' not found")]
    NamespaceNotFound(String),

    #[error("Group '{0}' already exists in namespace '{1}'")]
    GroupAlreadyExists(String, String),

    #[error("Group '{0}' not found in namespace '{1}'")]
    GroupNotFound(String, String),

    #[error("Value '{0}' already exists")]
    ValueAlreadyExists(String),

    #[error("Value '{0}' not found")]
    ValueNotFound(String),

    #[error("Invalid path format: {0}")]
    InvalidPath(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PathmapError>;
