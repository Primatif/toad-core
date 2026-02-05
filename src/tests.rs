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
    // Should be under 50ms
    assert!(
        duration.as_millis() < 50,
        "Fingerprinting too slow: {:?}",
        duration
    );
    Ok(())
}
