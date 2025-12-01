//! Configuration file loading and structures

use crate::cli::OutputFormat;
use anyhow::Result;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub format: Option<OutputFormat>,
    pub enable_native: Option<bool>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            format: Some(OutputFormat::Human),
            enable_native: Some(true),
        }
    }
}

pub fn load_config(path: Option<&Utf8PathBuf>) -> Result<ConfigFile> {
    let config_path = path
        .cloned()
        .unwrap_or_else(|| Utf8PathBuf::from(".dictate.toml"));

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(ConfigFile::default())
    }
}
