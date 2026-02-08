use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub mod strategy;

// --- Shared Data Models ---

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
}

// --- Tag Management ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagRegistry {
    /// Maps project name -> Set of tags
    pub projects: std::collections::HashMap<String, std::collections::HashSet<String>>,
}

impl TagRegistry {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_tag(&mut self, project: &str, tag: &str) {
        self.projects
            .entry(project.to_string())
            .or_default()
            .insert(tag.to_string());
    }

    pub fn remove_tag(&mut self, project: &str, tag: &str) {
        if let Some(tags) = self.projects.get_mut(project) {
            tags.remove(tag);
        }
    }

    pub fn get_tags(&self, project: &str) -> Vec<String> {
        self.projects
            .get(project)
            .map(|tags| {
                let mut t: Vec<String> = tags.iter().cloned().collect();
                t.sort();
                t
            })
            .unwrap_or_default()
    }
}

// --- Global Configuration ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub fingerprint: u64,
    pub projects: Vec<ProjectDetail>,
    pub last_sync: SystemTime,
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self {
            fingerprint: 0,
            projects: Vec::new(),
            last_sync: SystemTime::UNIX_EPOCH,
        }
    }
}

impl ProjectRegistry {
    pub fn registry_path(context_name: Option<&str>, base_dir: Option<&Path>) -> Result<PathBuf> {
        if let Some(name) = context_name {
            Ok(GlobalConfig::context_dir(name, base_dir)?.join("registry.json"))
        } else {
            Ok(GlobalConfig::config_dir(base_dir)?.join("registry.json"))
        }
    }

    pub fn load(context_name: Option<&str>, base_dir: Option<&Path>) -> Result<Self> {
        let path = Self::registry_path(context_name, base_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    pub fn save(&self, context_name: Option<&str>, base_dir: Option<&Path>) -> Result<()> {
        let path = Self::registry_path(context_name, base_dir)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub path: PathBuf,
    pub description: Option<String>,
    pub registered_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Absolute path to the anchored Toad workspace (legacy field)
    pub home_pointer: PathBuf,
    /// The name of the currently active project context
    pub active_context: Option<String>,
    /// All registered project contexts
    pub project_contexts: std::collections::HashMap<String, ProjectContext>,
}

impl GlobalConfig {
    /// Returns the base directory for Toad configuration.
    pub fn config_dir(base_dir: Option<&Path>) -> Result<PathBuf> {
        if let Some(base) = base_dir {
            return Ok(base.to_path_buf());
        }
        if let Ok(overridden) = std::env::var("TOAD_CONFIG_DIR") {
            return Ok(PathBuf::from(overridden));
        }
        dirs::home_dir()
            .map(|h| h.join(".toad"))
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
    }

    pub fn contexts_dir(base_dir: Option<&Path>) -> Result<PathBuf> {
        Ok(Self::config_dir(base_dir)?.join("contexts"))
    }

    pub fn context_dir(name: &str, base_dir: Option<&Path>) -> Result<PathBuf> {
        Ok(Self::contexts_dir(base_dir)?.join(name))
    }

    pub fn config_path(base_dir: Option<&Path>) -> Result<PathBuf> {
        Ok(Self::config_dir(base_dir)?.join("config.json"))
    }

    pub fn load(base_dir: Option<&Path>) -> Result<Option<Self>> {
        let path = Self::config_path(base_dir)?;
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let config_val: serde_json::Value = serde_json::from_str(&content)?;

        // Migration logic: If the config is in the old format (no active_context or project_contexts)
        if config_val.get("active_context").is_none()
            && config_val.get("project_contexts").is_none()
        {
            let home_pointer_val = config_val.get("home_pointer").and_then(|v| v.as_str());
            if let Some(home_path) = home_pointer_val {
                let path = PathBuf::from(home_path);
                let mut project_contexts = std::collections::HashMap::new();
                project_contexts.insert(
                    "default".to_string(),
                    ProjectContext {
                        path: path.clone(),
                        description: Some("Auto-migrated default context".to_string()),
                        registered_at: SystemTime::now(),
                    },
                );

                let migrated = Self {
                    home_pointer: path,
                    active_context: Some("default".to_string()),
                    project_contexts,
                };
                migrated.save(base_dir)?;
                migrated.migrate_legacy_artifacts(base_dir)?;
                return Ok(Some(migrated));
            }
        }

        let final_config: Self = serde_json::from_value(config_val)?;
        Ok(Some(final_config))
    }

    pub fn save(&self, base_dir: Option<&Path>) -> Result<()> {
        let dir = Self::config_dir(base_dir)?;
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(Self::config_path(base_dir)?, content)?;
        Ok(())
    }

    pub fn active_path(&self) -> Result<PathBuf> {
        if let Some(name) = &self.active_context
            && let Some(ctx) = self.project_contexts.get(name)
        {
            return Ok(ctx.path.clone());
        }
        Ok(self.home_pointer.clone())
    }

    /// Migrates registry.json and shadows/ from legacy locations to the new
    /// per-context directory (~/.toad/contexts/default/).
    pub fn migrate_legacy_artifacts(&self, base_dir: Option<&Path>) -> Result<()> {
        let config_dir = Self::config_dir(base_dir)?;
        let legacy_registry = config_dir.join("registry.json");
        let target_dir = Self::context_dir("default", base_dir)?;
        let target_shadows = target_dir.join("shadows");

        if legacy_registry.exists() || self.home_pointer.join("shadows").exists() {
            fs::create_dir_all(&target_shadows)?;

            // 1. Move registry.json
            if legacy_registry.exists() {
                let target_registry = target_dir.join("registry.json");
                if !target_registry.exists() {
                    fs::rename(&legacy_registry, &target_registry)?;
                    println!("Migrated registry.json to {:?}", target_registry);
                }
            }

            // 2. Move shadows/ content
            let legacy_shadows = self.home_pointer.join("shadows");
            if legacy_shadows.exists() && legacy_shadows.is_dir() {
                for entry in fs::read_dir(&legacy_shadows)? {
                    let entry = entry?;
                    let target_path = target_shadows.join(entry.file_name());
                    if !target_path.exists() {
                        fs::rename(entry.path(), &target_path)?;
                    }
                }
                // Attempt to remove the old directory if it's now empty
                let _ = fs::remove_dir(&legacy_shadows);
                println!(
                    "Migrated shadows from {:?} to {:?}",
                    legacy_shadows, target_shadows
                );
            }
        }

        Ok(())
    }
}

// --- Workspace Management ---

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub projects_dir: PathBuf,
    pub shadows_dir: PathBuf,
    pub active_context: Option<String>,
}

/// List of high-value metadata files monitored for context integrity.
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
    /// Attempts to discover the Toad workspace using 3-tier priority:
    /// 1. TOAD_ROOT env var
    /// 2. Upward search for .toad-root
    /// 3. Global config (~/.toad/config.json)
    pub fn discover() -> Result<Self> {
        // 1. Env Var
        if let Ok(env_root) = std::env::var("TOAD_ROOT") {
            let path = fs::canonicalize(PathBuf::from(env_root))?;
            return Ok(Self::with_root(path, None, None));
        }

        // 2. Local Upward Search
        if let Ok(cwd) = std::env::current_dir() {
            let mut curr = Some(cwd);
            while let Some(p) = curr {
                let canonical_p = fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
                if canonical_p.join(".toad-root").exists() {
                    return Ok(Self::with_root(canonical_p, None, None));
                }
                curr = p.parent().map(|parent| parent.to_path_buf());
            }
        }

        // 3. Global Config
        if let Some(config) = GlobalConfig::load(None)? {
            let path = config.active_path()?;
            if path.exists() {
                return Ok(Self::with_root(path, config.active_context, None));
            }
        }

        // Fallback: If no config exists but we are in a valid-looking dir, auto-initialize
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
                        ProjectContext {
                            path: root.clone(),
                            description: Some("Auto-initialized default context".to_string()),
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

    /// Generates a high-fidelity fingerprint of the managed projects.
    ///
    /// [IMPORTANT] This aggregation logic is order-dependent due to the rotational
    /// mixing algorithm. Project entries are sorted by name before processing to
    /// ensure deterministic results across different filesystem traversal orders.
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

        let mut entries: Vec<_> = fs::read_dir(&self.projects_dir)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();

        // Sort entries by name to ensure deterministic aggregation (required by mix logic)
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
            for file_name in HIGH_VALUE_FILES {
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

        // Include tags.json in fingerprint
        if let Ok(meta) = fs::metadata(self.tags_path()) {
            let mtime = meta
                .modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
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

#[cfg(test)]
mod tests;
