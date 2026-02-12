use crate::config::GlobalConfig;
use crate::error::ToadResult;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Workspace {
    /// The global Toad home directory (e.g., ~/.toad/)
    pub toad_home: PathBuf,
    /// The project directory for the current context (e.g., /path/to/my-code/)
    pub projects_dir: PathBuf,
    /// Metadata storage for the current context (e.g., ~/.toad/contexts/default/shadows/)
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
    pub fn discover() -> ToadResult<Self> {
        // Tier 1: TOAD_HOME env var
        let toad_home = if let Ok(env_home) = std::env::var("TOAD_HOME") {
            fs::canonicalize(PathBuf::from(env_home))?
        } else {
            GlobalConfig::config_dir(None)?
        };

        // Tier 2: TOAD_ROOT env var (explicit project override)
        if let Ok(env_root) = std::env::var("TOAD_ROOT") {
            let root_dir = fs::canonicalize(PathBuf::from(env_root))?;
            
            // Check if there is an active context in config to resolve shadows
            let config = GlobalConfig::load(None).ok().flatten();
            
            if let Some(config) = config {
                // Resolve base path from config, then check for projects/ subdirectory.
                // This mirrors migrate_legacy_to_global which returns root/projects/ when it exists.
                let base_dir = config.active_path()
                    .ok()
                    .filter(|p| p.exists())
                    .and_then(|p| fs::canonicalize(p).ok())
                    .unwrap_or(root_dir);
                let projects_dir = if base_dir.join("projects").exists() {
                    base_dir.join("projects")
                } else {
                    base_dir
                };
                let active_context = config.active_context;

                let shadows_dir = if let Some(name) = &active_context {
                    GlobalConfig::context_dir(name, None)?.join("shadows")
                } else {
                    toad_home.join("shadows")
                };

                return Ok(Self {
                    toad_home,
                    projects_dir,
                    shadows_dir,
                    active_context,
                });
            } else if root_dir.join(".toad-root").exists() {
                // If TOAD_ROOT points to a legacy workspace and no global config exists, migrate!
                let ws = Self::migrate_legacy_to_global(root_dir)?;
                return Ok(ws);
            } else {
                // TOAD_ROOT is explicitly set — honor it even without a config or .toad-root
                let projects_dir = if root_dir.join("projects").exists() {
                    root_dir.join("projects")
                } else {
                    root_dir
                };
                let shadows_dir = toad_home.join("shadows");
                return Ok(Self {
                    toad_home,
                    projects_dir,
                    shadows_dir,
                    active_context: None,
                });
            }
        }

        // Tier 3: Global Config
        if let Ok(Some(config)) = GlobalConfig::load(None) {
            let active_context_name = config.active_context.clone();
            let projects_dir = config.active_path().unwrap_or_else(|_| PathBuf::from("."));
            
            let projects_dir = if projects_dir.exists() {
                fs::canonicalize(projects_dir)?
            } else {
                projects_dir
            };

            let shadows_dir = if let Some(name) = &active_context_name {
                GlobalConfig::context_dir(name, None)?.join("shadows")
            } else {
                toad_home.join("shadows")
            };

            return Ok(Self {
                toad_home,
                projects_dir,
                shadows_dir,
                active_context: active_context_name,
            });
        }

        // Fallback: If no config, we might be in an uninitialized state
        Ok(Self {
            toad_home: toad_home.clone(),
            projects_dir: PathBuf::from("."),
            shadows_dir: toad_home.join("shadows"),
            active_context: None,
        })
    }

    fn migrate_legacy_to_global(legacy_root: PathBuf) -> ToadResult<Self> {
        let toad_home = GlobalConfig::config_dir(None)?;
        let context_name = "default";
        
        // 1. Create global config
        let mut project_contexts = std::collections::HashMap::new();
        project_contexts.insert(
            context_name.to_string(),
            crate::config::ProjectContext {
                path: legacy_root.clone(),
                description: Some("Auto-migrated legacy workspace".to_string()),
                context_type: if legacy_root.join(".gitmodules").exists() {
                    crate::config::ContextType::Hub
                } else {
                    crate::config::ContextType::Generic
                },
                ai_vendors: Vec::new(),
                registered_at: SystemTime::now(),
            },
        );

        let config = GlobalConfig {
            home_pointer: legacy_root.clone(),
            active_context: Some(context_name.to_string()),
            project_contexts,
            auto_sync: true,
            budget: crate::config::ContextBudget::default(),
        };
        config.save(None)?;

        // 2. Migrate artifacts
        let _ = config.migrate_legacy_artifacts(None)?;

        // 3. Return fresh workspace
        let projects_dir = if legacy_root.join("projects").exists() {
            legacy_root.join("projects")
        } else {
            legacy_root.clone()
        };

        Ok(Self {
            toad_home,
            projects_dir,
            shadows_dir: GlobalConfig::context_dir(context_name, None)?.join("shadows"),
            active_context: Some(context_name.to_string()),
        })
    }

    pub fn new() -> Self {
        Self::discover().unwrap_or_else(|_| {
             let home = dirs::home_dir().map(|h| h.join(".toad")).unwrap_or_else(|| PathBuf::from("."));
             Self {
                toad_home: home.clone(),
                projects_dir: PathBuf::from("."),
                shadows_dir: home.join("shadows"),
                active_context: None,
             }
        })
    }

    pub fn with_root(
        root: PathBuf,
        active_context: Option<String>,
        _base_dir: Option<&Path>,
    ) -> Self {
        let toad_home = GlobalConfig::config_dir(None).unwrap_or_else(|_| PathBuf::from("."));
        let shadows_dir = if let Some(name) = &active_context {
            GlobalConfig::context_dir(name, None)
                .map(|d| d.join("shadows"))
                .unwrap_or_else(|_| toad_home.join("shadows"))
        } else {
            toad_home.join("shadows")
        };

        Self {
            projects_dir: root,
            shadows_dir,
            toad_home,
            active_context,
        }
    }

    pub fn get_fingerprint(&self) -> ToadResult<u64> {
        let mut fingerprint: u64 = 0;

        fn mix(h: &mut u64, v: u64) {
            *h = h.wrapping_add(v);
            *h = h.rotate_left(13);
            *h = h.wrapping_mul(0x517cc1b727220a95);
        }

        if let Ok(meta) = fs::metadata(&self.projects_dir) {
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
            let file_path = self.projects_dir.join(file_name);
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

    pub fn ensure_shadows(&self) -> ToadResult<()> {
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

    pub fn context_json_path(&self) -> PathBuf {
        self.shadows_dir.join("context.json")
    }

    pub fn changelog_path(&self) -> PathBuf {
        self.shadows_dir.join("CHANGELOG.json")
    }

    pub fn stored_fingerprint(&self) -> u64 {
        crate::registry::ProjectRegistry::load(self.active_context.as_deref(), None)
            .map(|r| r.fingerprint)
            .unwrap_or(0)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
