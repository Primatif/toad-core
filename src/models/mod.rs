use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod changelog;

pub use changelog::{ChangeType, ProjectChange, EcosystemChangelog, ChangelogHistory};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StackStrategy {
    pub name: String,
    pub match_files: Vec<String>,
    pub artifacts: Vec<String>,
    pub tags: Vec<String>,
    pub priority: i32,
}

impl StackStrategy {
    pub fn matches(&self, files: &[String]) -> bool {
        self.match_files.iter().any(|m| files.contains(m))
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
pub struct SubmoduleDetail {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub stack: String,
    pub essence: Option<String>,
    pub taxonomy: Vec<String>,
    pub initialized: bool,
    pub vcs_status: VcsStatus,
    pub expected_commit: Option<String>,
    pub actual_commit: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TargetSource {
    HubRoot,
    Submodule,
    PondProject,
    Orphan,
}

impl std::fmt::Display for TargetSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HubRoot => write!(f, "HubRoot"),
            Self::Submodule => write!(f, "Submodule"),
            Self::PondProject => write!(f, "PondProject"),
            Self::Orphan => write!(f, "Orphan"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub name: String,
    pub path: PathBuf,
    pub stack: String,
    pub activity: ActivityTier,
    pub vcs_status: VcsStatus,
    pub essence: Option<String>,
    pub tags: Vec<String>,
    pub taxonomy: Vec<String>,
    pub artifact_dirs: Vec<String>,
    pub sub_projects: Vec<String>,
    pub submodules: Vec<SubmoduleDetail>,
    pub source: TargetSource,
    #[serde(default)]
    pub total_size: u64,
    #[serde(default)]
    pub bloat_index: f64,
}
