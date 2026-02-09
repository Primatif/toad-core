use crate::config::GlobalConfig;
use crate::models::ProjectDetail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

// --- Tag Management ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagRegistry {
    pub projects: std::collections::HashMap<String, std::collections::HashSet<String>>,
}

impl TagRegistry {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let registry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
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

// --- Project Registry ---

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
    pub fn registry_path(
        context_name: Option<&str>,
        base_dir: Option<&Path>,
    ) -> Result<std::path::PathBuf> {
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
