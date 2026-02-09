use crate::config::GlobalConfig;
use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub shadows_dir: PathBuf,
    pub active_context: Option<String>,
}

pub const HIGH_VALUE_FILES: &[&str] = &[
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

impl Workspace {
    pub fn discover() -> Result<Self> {
        if let Ok(env_root) = std::env::var("TOAD_ROOT") {
            let path = fs::canonicalize(PathBuf::from(env_root))?;
            return Ok(Self::with_root(path, None, None));
        }

        if let Ok(cwd) = std::env::current_dir() {
            let mut curr = Some(cwd);
            while let Some(p) = curr {
                let canonical_p = fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
                if canonical_p.join(".toad-root").exists() {
                    if let Ok(Some(config)) = GlobalConfig::load(None) {
                        for (name, ctx) in &config.project_contexts {
                            if ctx.path == canonical_p {
                                return Ok(Self::with_root(canonical_p, Some(name.clone()), None));
                            }
                        }
                    }
                    return Ok(Self::with_root(canonical_p, None, None));
                }
                curr = p.parent().map(|parent| parent.to_path_buf());
            }
        }

        if let Some(config) = GlobalConfig::load(None)? {
            let path = config.active_path()?;
            if path.exists() {
                return Ok(Self::with_root(path, config.active_context, None));
            }
        }

        if let Ok(cwd) = std::env::current_dir()
            && cwd.join(".toad-root").exists()
        {
            let root = fs::canonicalize(cwd)?;
            let config = GlobalConfig {
                home_pointer: root.clone(),
                active_context: Some("default".to_string()),
                project_contexts: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(
                        "default".to_string(),
                        crate::config::ProjectContext {
                            path: root.clone(),
                            description: Some("Auto-initialized default context".to_string()),
                            context_type: crate::config::ContextType::Generic,
                            ai_vendors: Vec::new(),
                            registered_at: SystemTime::now(),
                        },
                    );
                    m
                },
            };
            config.save(None)?;
            return Ok(Self::with_root(root, Some("default".to_string()), None));
        }

        bail!("Toad workspace not found. Use 'toad home <path>' to anchor a directory.")
    }

    pub fn new() -> Self {
        Self::discover().unwrap_or_else(|_| Self::with_root(PathBuf::from("."), None, None))
    }

    pub fn with_root(
        root: PathBuf,
        active_context: Option<String>,
        base_dir: Option<&Path>,
    ) -> Self {
        let shadows_dir = if let Some(name) = &active_context {
            GlobalConfig::context_dir(name, base_dir)
                .map(|d| d.join("shadows"))
                .unwrap_or_else(|_| root.join("shadows"))
        } else {
            root.join("shadows")
        };

        Self {
            projects_dir: root.join("projects"),
            shadows_dir,
            root,
            active_context,
        }
    }

    pub fn get_fingerprint(&self) -> Result<u64> {
        let mut fingerprint: u64 = 0;

        fn mix(h: &mut u64, v: u64) {
            *h = h.wrapping_add(v);
            *h = h.rotate_left(13);
            *h = h.wrapping_mul(0x517cc1b727220a95);
        }

        if let Ok(meta) = fs::metadata(&self.root) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            mix(&mut fingerprint, mtime);
        }

        if self.projects_dir.exists() {
            let mut entries: Vec<_> = fs::read_dir(&self.projects_dir)?
                .flatten()
                .filter(|e| e.path().is_dir())
                .collect();

            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let name_hash = name.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
                    mix(&mut fingerprint, name_hash);
                }

                if let Ok(meta) = fs::metadata(&path) {
                    let mtime = meta
                        .modified()
                        .ok()
                        .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    mix(&mut fingerprint, mtime);
                }

                for file_name in HIGH_VALUE_FILES {
                    let file_path = path.join(file_name);
                    if let Ok(meta) = fs::metadata(&file_path) {
                        let mtime = meta
                            .modified()
                            .ok()
                            .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        mix(&mut fingerprint, mtime);
                    }
                }
            }
        }

        for file_name in HIGH_VALUE_FILES {
            let file_path = self.root.join(file_name);
            if let Ok(meta) = fs::metadata(&file_path) {
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                mix(&mut fingerprint, mtime);
            }
        }

        if let Ok(meta) = fs::metadata(self.tags_path()) {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            mix(&mut fingerprint, mtime);
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

    pub fn tags_path(&self) -> PathBuf {
        self.shadows_dir.join("tags.json")
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
