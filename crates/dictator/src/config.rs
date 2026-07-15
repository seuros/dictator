//! Configuration file loading and structures

use crate::cli::OutputFormat;
use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::DictateConfig;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub format: Option<OutputFormat>,
    pub enable_native: Option<bool>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self { format: Some(OutputFormat::Human), enable_native: Some(true) }
    }
}

pub fn load_config(path: Option<&Utf8PathBuf>) -> Result<ConfigFile> {
    let config_path = path.cloned().unwrap_or_else(|| Utf8PathBuf::from(".dictate.toml"));

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(ConfigFile::default())
    }
}

pub fn load_dictate_config(
    path: Option<&Utf8PathBuf>,
    profile: Option<&str>,
) -> Result<Option<DictateConfig>> {
    let Some(config) = (if let Some(p) = path {
        Some(DictateConfig::from_file(p.as_std_path())?)
    } else {
        DictateConfig::load_default_strict()?
    }) else {
        return Ok(None);
    };

    let selected_profile = profile
        .or(config.active_profile.as_deref())
        .or_else(|| config.profile.contains_key("default").then_some("default"));

    let Some(profile_name) = selected_profile else {
        return Ok(Some(config));
    };

    let effective = config
        .get_profile_config(profile_name)
        .map_err(|e| anyhow::anyhow!("Profile error: {e}"))?;

    Ok(Some(effective))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(contents: &[u8]) -> (tempfile::NamedTempFile, Utf8PathBuf) {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, contents).unwrap();
        let path = Utf8PathBuf::from_path_buf(file.path().to_path_buf()).unwrap();
        (file, path)
    }

    #[test]
    fn load_dictate_config_applies_active_profile() {
        let (_file, path) = write_config(
            br#"
active_profile = "ci"

[decree.supreme]
max_line_length = 120

[profile.ci.decree.supreme]
max_line_length = 80
"#,
        );

        let config = load_dictate_config(Some(&path), None).unwrap().unwrap();

        assert_eq!(config.decree["supreme"].max_line_length, Some(80));
    }

    #[test]
    fn load_dictate_config_explicit_profile_overrides_active_profile() {
        let (_file, path) = write_config(
            br#"
active_profile = "ci"

[decree.supreme]
max_line_length = 120

[profile.ci.decree.supreme]
max_line_length = 80

[profile.relaxed.decree.supreme]
max_line_length = 140
"#,
        );

        let config = load_dictate_config(Some(&path), Some("relaxed")).unwrap().unwrap();

        assert_eq!(config.decree["supreme"].max_line_length, Some(140));
    }

    #[test]
    fn load_dictate_config_applies_default_profile_when_present() {
        let (_file, path) = write_config(
            br#"
[decree.supreme]
max_line_length = 120

[profile.default.decree.supreme]
max_line_length = 100
"#,
        );

        let config = load_dictate_config(Some(&path), None).unwrap().unwrap();

        assert_eq!(config.decree["supreme"].max_line_length, Some(100));
    }
}
