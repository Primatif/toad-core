use crate::{GlobalConfig, StackStrategy};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct StrategyRegistry {
    pub strategies: Vec<StackStrategy>,
}

impl StrategyRegistry {
    /// Loads all strategies from ~/.toad/strategies/builtin and ~/.toad/strategies/custom.
    /// If builtin is empty, it populates it with defaults.
    /// Custom strategies with the same filename as built-ins will replace them.
    pub fn load() -> Result<Self> {
        let config_dir = GlobalConfig::config_dir(None)?;
        let builtin_dir = config_dir.join("strategies/builtin");
        let custom_dir = config_dir.join("strategies/custom");

        // Ensure directories exist
        fs::create_dir_all(&builtin_dir)?;
        fs::create_dir_all(&custom_dir)?;

        // Populate builtins if empty
        if fs::read_dir(&builtin_dir)?.count() == 0 {
            Self::install_defaults(&builtin_dir)?;
        }

        let mut strategy_map = HashMap::new();

        // Load builtins
        for strategy in Self::load_map_from_dir(&builtin_dir)? {
            strategy_map.insert(strategy.0, strategy.1);
        }

        // Load custom (overwrites builtins with same filename)
        for strategy in Self::load_map_from_dir(&custom_dir)? {
            strategy_map.insert(strategy.0, strategy.1);
        }

        let mut strategies: Vec<StackStrategy> = strategy_map.into_values().collect();

        // Sort by priority (descending)
        strategies.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(Self { strategies })
    }

    fn load_map_from_dir(dir: &Path) -> Result<Vec<(String, StackStrategy)>> {
        let mut results = Vec::new();
        if !dir.exists() {
            return Ok(results);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let filename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let content = fs::read_to_string(&path)?;
                match toml::from_str::<StackStrategy>(&content) {
                    Ok(strategy) => results.push((filename, strategy)),
                    Err(e) => println!("WARNING: Failed to load strategy at {:?}: {}", path, e),
                }
            }
        }
        Ok(results)
    }

    pub fn load_from_dir(dir: &Path) -> Result<Vec<StackStrategy>> {
        let results = Self::load_map_from_dir(dir)?;
        let mut strategies: Vec<StackStrategy> = results.into_iter().map(|(_, s)| s).collect();
        strategies.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(strategies)
    }

    pub fn install_defaults(dir: &Path) -> Result<()> {
        let rust = r##"name = "Rust"
match_files = ["Cargo.toml"]
artifacts = ["target"]
tags = ["#rust"]
priority = 10
"##;
        let node = r##"name = "NodeJS"
match_files = ["package.json"]
artifacts = ["node_modules", "dist", ".next", "build", "out"]
tags = ["#nodejs"]
priority = 10
"##;
        let go = r##"name = "Go"
match_files = ["go.mod"]
artifacts = ["bin", "vendor"]
tags = ["#go"]
priority = 10
"##;
        let python = r##"name = "Python"
match_files = ["requirements.txt", "pyproject.toml"]
artifacts = ["__pycache__", ".venv", "venv", ".pytest_cache", "build", "dist"]
tags = ["#python"]
priority = 10
"##;
        let monorepo = r##"name = "Monorepo"
match_files = ["nx.json", "turbo.json", "go.work", "lerna.json"]
artifacts = ["node_modules", "target", ".turbo", "dist"]
tags = ["#monorepo"]
priority = 20
"##;
        let docker = r##"name = "Docker"
match_files = ["Dockerfile"]
artifacts = []
tags = ["#docker"]
priority = 5
"##;
        let tauri = r##"name = "Tauri"
match_files = ["tauri.conf.json"]
artifacts = ["src-tauri/target", "src-tauri/bin"]
tags = ["#tauri", "#desktop"]
priority = 15
"##;
        let wails = r##"name = "Wails"
match_files = ["wails.json", "Wails.json"]
artifacts = ["build/bin", "frontend/dist"]
tags = ["#wails", "#desktop"]
priority = 15
"##;

        fs::write(dir.join("rust.toml"), rust)?;
        fs::write(dir.join("node.toml"), node)?;
        fs::write(dir.join("go.toml"), go)?;
        fs::write(dir.join("python.toml"), python)?;
        fs::write(dir.join("monorepo.toml"), monorepo)?;
        fs::write(dir.join("docker.toml"), docker)?;
        fs::write(dir.join("tauri.toml"), tauri)?;
        fs::write(dir.join("wails.toml"), wails)?;

        Ok(())
    }
}
