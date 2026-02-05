use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

// --- Shared Data Models ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStack {
    Rust,
    Go,
    NodeJS,
    Python,
    Monorepo,
    Generic,
}

impl std::fmt::Display for ProjectStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Go => write!(f, "Go"),
            Self::NodeJS => write!(f, "NodeJS"),
            Self::Python => write!(f, "Python"),
            Self::Monorepo => write!(f, "Monorepo"),
            Self::Generic => write!(f, "Generic"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityTier {
    Active,
    Cold,
    Archive,
}

impl std::fmt::Display for ActivityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "🔥 Active"),
            Self::Cold => write!(f, "❄️ Cold"),
            Self::Archive => write!(f, "🗄️ Archive"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VcsStatus {
    Clean,
    Dirty,
    Untracked,
    None,
}

impl std::fmt::Display for VcsStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clean => write!(f, "✅ Clean"),
            Self::Dirty => write!(f, "⚠️ Dirty"),
            Self::Untracked => write!(f, "❓ Untracked"),
            Self::None => write!(f, "N/A"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub name: String,
    pub path: PathBuf,
    pub stack: ProjectStack,
    pub activity: ActivityTier,
    pub vcs_status: VcsStatus,
    pub essence: Option<String>,
    pub hashtags: Vec<String>,
    pub sub_projects: Vec<String>,
}

// --- Workspace Management ---

pub struct Workspace {
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub shadows_dir: PathBuf,
}

impl Workspace {
    pub fn new() -> Self {
        Self::with_root(PathBuf::from("."))
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self {
            projects_dir: root.join("projects"),
            shadows_dir: root.join("shadows"),
            root,
        }
    }

    pub fn get_fingerprint(&self) -> Result<u64> {
        let metadata = fs::metadata(&self.projects_dir).context("Failed to stat projects root")?;
        let mtime = metadata
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        Ok(mtime)
    }

    pub fn ensure_shadows(&self) -> Result<()> {
        if !self.shadows_dir.exists() {
            fs::create_dir_all(&self.shadows_dir)?;
        }
        Ok(())
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.shadows_dir.join("MANIFEST.md")
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;