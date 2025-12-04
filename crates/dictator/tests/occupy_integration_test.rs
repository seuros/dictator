//! Integration tests for the occupy command

use anyhow::Result;
use camino::Utf8PathBuf;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_occupy_creates_config_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;

    let config_path = temp_path.join(".dictate.toml");
    assert!(!config_path.exists());

    // Run occupy command
    dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: temp_path.clone(),
        force: false,
    })?;

    // Verify file was created
    assert!(config_path.exists());

    // Verify content is valid TOML
    let content = fs::read_to_string(&config_path)?;
    let _parsed: toml::Value = toml::from_str(&content)?;

    Ok(())
}

#[test]
fn test_occupy_fails_without_force_if_exists() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;

    let config_path = temp_path.join(".dictate.toml");

    // Create existing file
    fs::write(&config_path, "# existing content\n")?;

    // Attempt occupy without --force should fail
    let result = dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: temp_path.clone(),
        force: false,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    // Original content should be preserved
    let content = fs::read_to_string(&config_path)?;
    assert_eq!(content, "# existing content\n");

    Ok(())
}

#[test]
fn test_occupy_overwrites_with_force() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;

    let config_path = temp_path.join(".dictate.toml");

    // Create existing file
    fs::write(&config_path, "# old content\n")?;

    // Occupy with --force should succeed
    dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: temp_path.clone(),
        force: true,
    })?;

    // Content should be replaced
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("decree.supreme"));
    assert!(!content.contains("# old content"));

    Ok(())
}

#[test]
fn test_occupy_fails_on_nonexistent_directory() {
    let nonexistent = Utf8PathBuf::from("/nonexistent/path/that/does/not/exist");

    let result = dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: nonexistent,
        force: false,
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_occupy_creates_valid_dictate_config() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;

    dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: temp_path.clone(),
        force: false,
    })?;

    let config_path = temp_path.join(".dictate.toml");

    // Should parse as DictateConfig
    let config = dictator_core::DictateConfig::from_file(config_path.as_std_path())?;

    // Only supreme decree is active by default (language-specific decrees are commented out)
    assert!(config.decree.contains_key("supreme"));

    // Language-specific decrees should NOT be present (they're commented out for opt-in)
    assert!(!config.decree.contains_key("ruby"));
    assert!(!config.decree.contains_key("typescript"));
    assert!(!config.decree.contains_key("golang"));
    assert!(!config.decree.contains_key("rust"));
    assert!(!config.decree.contains_key("python"));
    assert!(!config.decree.contains_key("frontmatter"));

    Ok(())
}

#[test]
fn test_occupy_with_current_directory_default() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("non-utf8 path"))?;

    // Change to temp directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_path)?;

    // Run occupy with default path (".")
    let result = dictator::occupy::run_occupy(dictator::cli::OccupyArgs {
        path: Utf8PathBuf::from("."),
        force: false,
    });

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    result?;

    // Verify file was created
    let config_path = temp_path.join(".dictate.toml");
    assert!(config_path.exists());

    Ok(())
}
