//! Integration test for the occupy command — the full lifecycle in one pass.

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator::cli::OccupyArgs;
use dictator::occupy::run_occupy;
use std::fs;
use tempfile::TempDir;

#[test]
fn occupy_lifecycle() -> Result<()> {
    // Nonexistent directory is rejected.
    let result = run_occupy(OccupyArgs {
        path: Utf8PathBuf::from("/nonexistent/path/that/does/not/exist"),
        force: false,
    });
    assert!(result.unwrap_err().to_string().contains("does not exist"));

    // Fresh directory: config is created and parses as a valid DictateConfig.
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;
    let config_path = temp_path.join(".dictate.toml");
    assert!(!config_path.exists());

    run_occupy(OccupyArgs { path: temp_path.clone(), force: false })?;
    assert!(config_path.exists());
    let _parsed: toml::Value = toml::from_str(&fs::read_to_string(&config_path)?)?;

    let config = dictator_core::DictateConfig::from_file(config_path.as_std_path())?;
    // Only the supreme decree is active by default; language decrees ship commented out.
    assert!(config.decree.contains_key("supreme"));
    for lang in ["ruby", "typescript", "golang", "rust", "python", "frontmatter"] {
        assert!(!config.decree.contains_key(lang), "{lang} should be opt-in");
    }

    // Existing config without --force is refused and left untouched.
    fs::write(&config_path, "# existing content\n")?;
    let result = run_occupy(OccupyArgs { path: temp_path.clone(), force: false });
    assert!(result.unwrap_err().to_string().contains("already exists"));
    assert_eq!(fs::read_to_string(&config_path)?, "# existing content\n");

    // --force overwrites.
    run_occupy(OccupyArgs { path: temp_path.clone(), force: true })?;
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("decree.supreme"));
    assert!(!content.contains("# existing content"));

    // Default path "." resolves to the current directory. Safe to chdir here:
    // this is the only test in the binary, so nothing runs in parallel with it.
    let cwd_dir = TempDir::new()?;
    let cwd_path = Utf8PathBuf::from_path_buf(cwd_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&cwd_path)?;
    let result = run_occupy(OccupyArgs { path: Utf8PathBuf::from("."), force: false });
    let restore_result = std::env::set_current_dir(original_dir);
    result?;
    restore_result?;
    assert!(cwd_path.join(".dictate.toml").exists());

    Ok(())
}
