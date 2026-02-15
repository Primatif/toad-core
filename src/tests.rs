use super::*;
use crate::config::ContextBudget;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use tempfile::tempdir;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_workspace_paths() {
    let home = PathBuf::from("/tmp/toad_home");
    let projects = PathBuf::from("/tmp/projects");
    let ws = Workspace {
        toad_home: home.clone(),
        projects_dir: projects.clone(),
        shadows_dir: home.join("shadows"),
        active_context: None,
    };
    assert_eq!(ws.projects_dir, projects);
    assert_eq!(ws.shadows_dir, home.join("shadows"));
    assert_eq!(ws.manifest_path(), home.join("shadows").join("MANIFEST.md"));
}

#[test]
fn test_ensure_shadows() -> Result<()> {
    let dir = tempdir()?;
    let home = dir.path().join(".toad");
    let ws = Workspace {
        toad_home: home.clone(),
        projects_dir: PathBuf::from("."),
        shadows_dir: home.join("shadows"),
        active_context: None,
    };

    assert!(!ws.shadows_dir.exists());
    ws.ensure_shadows()?;
    assert!(ws.shadows_dir.exists());
    Ok(())
}

#[test]
fn test_get_fingerprint() -> Result<()> {
    let dir = tempdir()?;
    let projects_dir = dir.path().join("projects");
    fs::create_dir(&projects_dir)?;

    let ws = Workspace {
        toad_home: dir.path().join(".toad"),
        projects_dir: projects_dir.clone(),
        shadows_dir: dir.path().join(".toad/shadows"),
        active_context: None,
    };

    let fp1 = ws.get_fingerprint()?;
    assert!(fp1 > 0);

    // Create a project
    let proj_dir = projects_dir.join("test-proj");
    fs::create_dir(&proj_dir)?;

    // Explicitly set mtime to be different
    let future = filetime::FileTime::from_system_time(
        SystemTime::now() + std::time::Duration::from_secs(10),
    );
    filetime::set_file_mtime(&proj_dir, future)?;

    let fp2 = ws.get_fingerprint()?;
    assert_ne!(fp1, fp2, "Fingerprint should change when project is added");

    Ok(())
}

#[test]
fn test_fingerprint_performance() -> Result<()> {
    let dir = tempdir()?;
    let projects_dir = dir.path().join("projects");
    fs::create_dir(&projects_dir)?;

    let ws = Workspace {
        toad_home: dir.path().join(".toad"),
        projects_dir: projects_dir.clone(),
        shadows_dir: dir.path().join(".toad/shadows"),
        active_context: None,
    };

    // Create 100 projects with 5 high-value files each
    for i in 0..100 {
        let proj_dir = projects_dir.join(format!("proj-{}", i));
        fs::create_dir(&proj_dir)?;
        fs::write(proj_dir.join("README.md"), "test")?;
        fs::write(proj_dir.join("Cargo.toml"), "test")?;
        fs::write(proj_dir.join("package.json"), "test")?;
        fs::write(proj_dir.join(".gitignore"), "test")?;
        fs::create_dir(proj_dir.join(".git"))?;
        fs::write(proj_dir.join(".git/index"), "test")?;
    }

    let start = std::time::Instant::now();
    let _fp = ws.get_fingerprint()?;
    let duration = start.elapsed();

    println!("Fingerprinting 100 projects took: {:?}", duration);
    // Should be under 100ms (raised from 50ms for CI variance)
    assert!(
        duration.as_millis() < 100,
        "Fingerprinting too slow: {:?}",
        duration
    );
    Ok(())
}

#[test]
fn test_stack_strategy_serialization() -> Result<()> {
    let strategy = StackStrategy {
        name: "Rust".to_string(),
        match_files: vec!["Cargo.toml".to_string()],
        artifacts: vec!["target".to_string()],
        tags: vec!["#rust".to_string()],
        priority: 10,
    };

    let toml = toml::to_string(&strategy)?;
    assert!(toml.contains("name = \"Rust\""));
    assert!(toml.contains("match_files = [\"Cargo.toml\"]"));

    let loaded: StackStrategy = toml::from_str(&toml)?;
    assert_eq!(strategy, loaded);
    Ok(())
}

#[test]
fn test_strategy_registry_embedded_and_custom() -> Result<()> {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Mock config dir
    let dir = tempdir()?;
    let config_path = fs::canonicalize(dir.path())?;
    unsafe {
        std::env::set_var("TOAD_CONFIG_DIR", config_path.to_str().unwrap());
    }

    // 1. Test embedded defaults
    let registry = crate::strategy::StrategyRegistry::load()?;
    assert!(registry.strategies.len() >= 8);
    assert!(registry.strategies.iter().any(|s| s.name == "Rust"));

    // 2. Test custom override
    let custom_dir = dir.path().join("strategies/custom");
    fs::create_dir_all(&custom_dir)?;

    let rust_override = r##"name = "Rust Custom"
match_files = ["Cargo.toml"]
artifacts = ["custom_target"]
tags = ["#rust", "#custom"]
priority = 100
"##;
    fs::write(custom_dir.join("rust.toml"), rust_override)?;

    let registry2 = crate::strategy::StrategyRegistry::load()?;
    let rust = registry2
        .strategies
        .iter()
        .find(|s| s.match_files.contains(&"Cargo.toml".to_string()))
        .unwrap();
    assert_eq!(rust.name, "Rust Custom");
    assert_eq!(rust.priority, 100);
    assert_eq!(rust.priority, 100);

    unsafe {
        std::env::remove_var("TOAD_CONFIG_DIR");
    }
    Ok(())
}

#[test]
fn test_strategy_registry_load_from_dir() -> Result<()> {
    let dir = tempdir()?;
    let rust_toml = r##"name = "Rust"
match_files = ["Cargo.toml"]
artifacts = ["target"]
tags = ["#rust"]
priority = 10
"##;
    fs::write(dir.path().join("rust.toml"), rust_toml)?;

    let strategies = crate::strategy::StrategyRegistry::load_from_dir(dir.path())?;
    assert_eq!(strategies.len(), 1);
    assert_eq!(strategies[0].name, "Rust");

    Ok(())
}

#[test]
fn test_project_registry_serialization() -> Result<()> {
    let mut registry = ProjectRegistry::default();
    registry.fingerprint = 12345;
    registry.projects.push(ProjectDetail {
        name: "test-proj".to_string(),
        path: PathBuf::from("/tmp/test-proj"),
        stack: "Rust".to_string(),
        activity: ActivityTier::Active,
        vcs_status: VcsStatus::Clean,
        essence: Some("A test project".to_string()),
        tags: vec!["#tag1".to_string()],
        taxonomy: vec!["#rust".to_string(), "#test".to_string()],
        artifact_dirs: vec!["target".to_string()],
        sub_projects: vec![],
        submodules: vec![],
        source: TargetSource::PondProject,
        total_size: 0,
        bloat_index: 0.0,
        dna: ProjectDna::default(),
    });

    // Mock the config dir for testing
    let dir = tempdir()?;
    let registry_path = dir.path().join("registry.json");

    // We can't easily override registry_path() without changing the code
    // So we just test manual save/load logic using the same serde logic
    let content = serde_json::to_string_pretty(&registry)?;
    fs::write(&registry_path, content)?;

    let loaded_content = fs::read_to_string(&registry_path)?;
    let loaded: ProjectRegistry = serde_json::from_str(&loaded_content)?;

    assert_eq!(loaded.fingerprint, 12345);
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(loaded.projects[0].name, "test-proj");

    Ok(())
}

#[test]
fn test_workspace_discovery_tiers() -> Result<()> {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Mock config dir to avoid real ~/.toad
    let config_dir = tempdir()?;
    let config_path = fs::canonicalize(config_dir.path())?;
    unsafe {
        std::env::set_var("TOAD_CONFIG_DIR", config_path.to_str().unwrap());
    }

    // 1. Env Var tier (TOAD_ROOT)
    let projects_root = tempdir()?;
    let projects_root_path = fs::canonicalize(projects_root.path())?;
    unsafe {
        std::env::set_var("TOAD_ROOT", projects_root_path.to_str().unwrap());
    }
    let ws = Workspace::discover()?;
    assert_eq!(fs::canonicalize(&ws.projects_dir)?, projects_root_path);
    assert_eq!(
        fs::canonicalize(&ws.toad_home)?,
        fs::canonicalize(&config_path)?
    );

    unsafe {
        std::env::remove_var("TOAD_ROOT");
    }

    // 2. Legacy Upward Migration tier
    let sandbox = tempdir()?;
    let legacy_root = fs::canonicalize(sandbox.path())?.join("my-legacy-proj");
    fs::create_dir_all(&legacy_root)?;
    fs::write(legacy_root.join(".toad-root"), "legacy")?;
    fs::create_dir(legacy_root.join("shadows"))?;
    fs::write(legacy_root.join("shadows/tags.json"), "{}")?;

    // Fresh config dir for this part
    let config_dir2 = tempdir()?;
    let config_path2 = fs::canonicalize(config_dir2.path())?;
    unsafe {
        std::env::set_var("TOAD_CONFIG_DIR", config_path2.to_str().unwrap());
    }

    let original_cwd = std::env::current_dir()?;
    std::env::set_current_dir(&legacy_root)?;
    unsafe {
        std::env::set_var("TOAD_ROOT", legacy_root.to_str().unwrap());
    }

    let ws2 = Workspace::discover()?;
    assert_eq!(
        fs::canonicalize(&ws2.projects_dir)?,
        fs::canonicalize(&legacy_root)?
    );
    assert_eq!(ws2.active_context, Some("default".to_string()));

    // Check if global config was created
    assert!(config_path2.join("config.json").exists());
    // Check if shadows were migrated to the context dir
    assert!(
        config_path2
            .join("contexts/default/shadows/tags.json")
            .exists()
    );

    std::env::set_current_dir(original_cwd)?;
    unsafe {
        std::env::remove_var("TOAD_ROOT");
    }

    unsafe {
        std::env::remove_var("TOAD_CONFIG_DIR");
    }

    Ok(())
}

#[test]
fn test_global_config_persistence() -> Result<()> {
    let dir = tempdir()?;
    let config_dir = dir.path().to_path_buf();

    let config = GlobalConfig {
        home_pointer: PathBuf::from("/tmp/fake"),
        active_context: Some("default".to_string()),
        project_contexts: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "default".to_string(),
                ProjectContext {
                    path: PathBuf::from("/tmp/fake"),
                    description: None,
                    context_type: ContextType::Generic,
                    ai_vendors: Vec::new(),
                    registered_at: SystemTime::now(),
                },
            );
            m
        },
        auto_sync: true,
        budget: ContextBudget::default(),
    };
    config.save(Some(&config_dir))?;

    let loaded = GlobalConfig::load(Some(&config_dir))?.expect("Config should be loaded");
    assert_eq!(loaded.home_pointer, PathBuf::from("/tmp/fake"));
    assert!(loaded.auto_sync);
    assert_eq!(loaded.budget.ecosystem_tokens, 2000);

    Ok(())
}
