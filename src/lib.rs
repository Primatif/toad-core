use anyhow::{bail, Result};
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
        if !self.projects_dir.exists() {
            bail!("Projects directory does not exist");
        }

        let mut fingerprint: u64 = 0;

        fn mix(h: &mut u64, v: u64) {
            *h = h.wrapping_add(v);
            *h = h.rotate_left(13);
            *h = h.wrapping_mul(0x517cc1b727220a95);
        }

        // Level 1: Root directory mtime
        let root_meta = fs::metadata(&self.projects_dir)?;
        let root_mtime = root_meta
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        mix(&mut fingerprint, root_mtime);

        // Level 2 & 3: Scan projects and high-value files
        let high_value_files = [
            "Cargo.toml",
            "Cargo.lock",
            "package.json",
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "go.mod",
            "go.sum",
            "go.work",
            "pyproject.toml",
            "requirements.txt",
            "poetry.lock",
            "README.md",
            "README.markdown",
            "readme.md",
            "Justfile",
            ".gitignore",
            ".git/index",
        ];

        let mut entries: Vec<_> = fs::read_dir(&self.projects_dir)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();
        
        // Sort entries by name to ensure deterministic aggregation
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let name_hash = name.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
                mix(&mut fingerprint, name_hash);
            }

            // Project dir mtime
            if let Ok(meta) = fs::metadata(&path) {
                let mtime = meta
                    .modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                mix(&mut fingerprint, mtime);
            }

            // Scan high-value files
            for file_name in &high_value_files {
                let file_path = path.join(file_name);
                if let Ok(meta) = fs::metadata(&file_path) {
                    let mtime = meta
                        .modified()?
                        .duration_since(SystemTime::UNIX_EPOCH)?
                        .as_secs();
                    mix(&mut fingerprint, mtime);
                }
            }
        }

        Ok(fingerprint)
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
