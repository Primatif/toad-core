use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToadError {
    #[error("Workspace not found. Use 'toad home <path>' to anchor a directory.")]
    WorkspaceNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Toml error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Path does not exist: {0:?}")]
    PathNotFound(PathBuf),

    #[error("Context already exists: {0}")]
    ContextExists(String),

    #[error("Context not found: {0}")]
    ContextNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Other error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    Other(String),
}

pub type ToadResult<T> = std::result::Result<T, ToadError>;
