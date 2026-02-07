use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_workspace_paths() {
    let root = PathBuf::from("/tmp/toad");
    let ws = Workspace::with_root(root.clone());
    assert_eq!(ws.projects_dir, root.join("projects"));
    assert_eq!(ws.shadows_dir, root.join("shadows"));
    assert_eq!(ws.manifest_path(), root.join("shadows").join("MANIFEST.md"));
}

#[test]
fn test_ensure_shadows() -> Result<()> {
    let dir = tempdir()?;
    let ws = Workspace::with_root(dir.path().to_path_buf());

    assert!(!ws.shadows_dir.exists());
    ws.ensure_shadows()?;
    assert!(ws.shadows_dir.exists());
    Ok(())
}

#[test]
fn test_get_fingerprint() -> Result<()> {
    let dir = tempdir()?;
    let ws = Workspace::with_root(dir.path().to_path_buf());

    // Should fail if projects dir doesn't exist
    assert!(ws.get_fingerprint().is_err());

    fs::create_dir(&ws.projects_dir)?;
    let fp1 = ws.get_fingerprint()?;
    assert!(fp1 > 0);

    // Create a project
    let proj_dir = ws.projects_dir.join("test-proj");
    fs::create_dir(&proj_dir)?;

    // Explicitly set mtime to be different from root
    let future = filetime::FileTime::from_system_time(
        SystemTime::now() + std::time::Duration::from_secs(10),
    );
    filetime::set_file_mtime(&proj_dir, future)?;

    let fp2 = ws.get_fingerprint()?;
    assert_ne!(fp1, fp2, "Fingerprint should change when project is added");

    // Add a high-value file
    let readme_path = proj_dir.join("README.md");
    fs::write(&readme_path, "hello")?;

    let even_further = filetime::FileTime::from_system_time(
        SystemTime::now() + std::time::Duration::from_secs(20),
    );
    filetime::set_file_mtime(&readme_path, even_further)?;

    let fp3 = ws.get_fingerprint()?;
    assert_ne!(
        fp2, fp3,
        "Fingerprint should change when README is added/modified"
    );

    Ok(())
}

#[test]
fn test_fingerprint_performance() -> Result<()> {
    let dir = tempdir()?;
    let ws = Workspace::with_root(dir.path().to_path_buf());
    fs::create_dir(&ws.projects_dir)?;

    // Create 100 projects with 5 high-value files each
    for i in 0..100 {
        let proj_dir = ws.projects_dir.join(format!("proj-{}", i));
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
fn test_strategy_registry_install_and_load() -> Result<()> {
    let dir = tempdir()?;
    let builtin_dir = dir.path().join("builtin");
    fs::create_dir(&builtin_dir)?;

    crate::strategy::StrategyRegistry::install_defaults(&builtin_dir)?;

    let strategies = crate::strategy::StrategyRegistry::load_from_dir(&builtin_dir)?;
    assert!(strategies.len() >= 5);

    let rust = strategies
        .iter()
        .find(|s| s.name == "Rust")
        .expect("Rust strategy missing");
    assert_eq!(rust.match_files, vec!["Cargo.toml".to_string()]);
    assert_eq!(rust.artifacts, vec!["target".to_string()]);

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
