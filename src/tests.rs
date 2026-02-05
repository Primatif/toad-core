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
    let fp = ws.get_fingerprint()?;
    assert!(fp > 0);
    Ok(())
}
