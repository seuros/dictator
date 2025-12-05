//! Occupy command - initialize .dictate.toml with defaults

use anyhow::Result;
use camino::Utf8PathBuf;
use std::fs;

use crate::cli::OccupyArgs;

/// Default .dictate.toml template with sensible settings
const DEFAULT_CONFIG: &str = include_str!("../templates/default.dictate.toml");

/// Run the occupy command to initialize a .dictate.toml file.
///
/// # Errors
///
/// Returns an error if:
/// - The target path is not a valid UTF-8 path
/// - The target directory does not exist
/// - The target path is not a directory
/// - The config file already exists and `--force` is not set
/// - Writing the config file fails
pub fn run_occupy(args: OccupyArgs) -> Result<()> {
    let target_dir = if args.path.is_absolute() {
        args.path
    } else {
        let cwd = std::env::current_dir()?;
        Utf8PathBuf::from_path_buf(cwd)
            .map_err(|_| anyhow::anyhow!("non-utf8 path"))?
            .join(&args.path)
    };

    // Ensure target directory exists
    if !target_dir.exists() {
        return Err(anyhow::anyhow!(
            "Target directory does not exist: {target_dir}"
        ));
    }

    if !target_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "Target path is not a directory: {target_dir}"
        ));
    }

    let config_path = target_dir.join(".dictate.toml");
    let cache_dir = target_dir.join(".dictator").join("cache");
    let gitignore_path = target_dir.join(".gitignore");

    // Ensure cache directory exists (per-worktree) and is user-private on Unix
    std::fs::create_dir_all(&cache_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&cache_dir, std::fs::Permissions::from_mode(0o700));
    }

    // Check if file exists
    if config_path.exists() && !args.force {
        return Err(anyhow::anyhow!(
            ".dictate.toml already exists at {config_path}\nUse --force to overwrite"
        ));
    }

    // Write the default config
    fs::write(&config_path, DEFAULT_CONFIG)?;

    println!("✓ Created .dictate.toml at {config_path}");

    // Ensure .dictator/ is gitignored (append if missing)
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.lines().any(|l| l.trim() == ".dictator/") {
            let mut updated = content;
            if !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(".dictator/\n");
            fs::write(&gitignore_path, updated)?;
            println!("✓ Updated .gitignore to ignore .dictator/");
        }
    } else {
        fs::write(&gitignore_path, ".dictator/\n")?;
        println!("✓ Created .gitignore ignoring .dictator/");
    }

    println!("✓ Ensured cache dir at {}", cache_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid_toml() {
        // Ensure the default config parses as valid TOML
        let parsed: Result<toml::Value, _> = toml::from_str(DEFAULT_CONFIG);
        assert!(
            parsed.is_ok(),
            "Default config must be valid TOML: {:?}",
            parsed.err()
        );
    }

    #[test]
    fn test_default_config_has_supreme_decree() {
        let config: toml::Value = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert!(
            config
                .get("decree")
                .and_then(|d| d.get("supreme"))
                .is_some(),
            "Default config must include decree.supreme"
        );
    }

    #[test]
    fn test_default_config_structure() {
        // Validate that config can be deserialized into DictateConfig
        use dictator_core::DictateConfig;
        let config: Result<DictateConfig, _> = toml::from_str(DEFAULT_CONFIG);
        assert!(
            config.is_ok(),
            "Default config must match DictateConfig structure: {:?}",
            config.err()
        );
    }
}
