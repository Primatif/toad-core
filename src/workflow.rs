use crate::config::GlobalConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub script_path: PathBuf,
    pub registered_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowRegistry {
    pub workflows: std::collections::HashMap<String, CustomWorkflow>,
    pub reserved_namespaces: Vec<String>,
}

impl WorkflowRegistry {
    pub fn registry_path(base_dir: Option<&Path>) -> Result<PathBuf> {
        Ok(GlobalConfig::config_dir(base_dir)?.join("workflows.json"))
    }

    pub fn load(base_dir: Option<&Path>) -> Result<Self> {
        let path = Self::registry_path(base_dir)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, base_dir: Option<&Path>) -> Result<()> {
        let path = Self::registry_path(base_dir)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_reserved(&self, name: &str) -> bool {
        self.reserved_namespaces.contains(&name.to_string())
    }
}
