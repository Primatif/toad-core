use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContextType {
    Hub,
    Pond,
    Generic,
}

impl std::fmt::Display for ContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub => write!(f, "Hub"),
            Self::Pond => write!(f, "Pond"),
            Self::Generic => write!(f, "Generic"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub path: PathBuf,
    pub description: Option<String>,
    pub context_type: ContextType,
    #[serde(default)]
    pub ai_vendors: Vec<String>,
    pub registered_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub home_pointer: PathBuf,
    pub active_context: Option<String>,
    pub project_contexts: std::collections::HashMap<String, ProjectContext>,
}

impl GlobalConfig {
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
                        context_type: ContextType::Generic,
                        ai_vendors: Vec::new(),
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

    pub fn migrate_legacy_artifacts(&self, base_dir: Option<&Path>) -> Result<()> {
        let config_dir = Self::config_dir(base_dir)?;
        let legacy_registry = config_dir.join("registry.json");
        let target_dir = Self::context_dir("default", base_dir)?;
        let target_shadows = target_dir.join("shadows");

        if legacy_registry.exists() || self.home_pointer.join("shadows").exists() {
            fs::create_dir_all(&target_shadows)?;

            if legacy_registry.exists() {
                let target_registry = target_dir.join("registry.json");
                if !target_registry.exists() {
                    fs::rename(&legacy_registry, &target_registry)?;
                    println!("Migrated registry.json to {:?}", target_registry);
                }
            }

            let legacy_shadows = self.home_pointer.join("shadows");
            if legacy_shadows.exists() && legacy_shadows.is_dir() {
                for entry in fs::read_dir(&legacy_shadows)? {
                    let entry = entry?;
                    let target_path = target_shadows.join(entry.file_name());
                    if !target_path.exists() {
                        fs::rename(entry.path(), &target_path)?;
                    }
                }
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
