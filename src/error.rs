use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ToadError {
    #[error("Workspace not found. Use 'toad home <path>' to anchor a directory.")]
    WorkspaceNotFound,

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serde(String),

    #[error("Toml error: {0}")]
    Toml(String),

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
    Anyhow(String),

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for ToadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for ToadError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}

impl From<toml::de::Error> for ToadError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e.to_string())
    }
}

impl From<anyhow::Error> for ToadError {
    fn from(e: anyhow::Error) -> Self {
        Self::Anyhow(e.to_string())
    }
}

pub type ToadResult<T> = std::result::Result<T, ToadError>;
