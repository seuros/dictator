//! Regime initialization and decree loading

use dictator_core::{DictateConfig, Regime};
use dictator_frontmatter::init_decree as create_frontmatter_plugin;
use dictator_golang::init_decree as create_golang_plugin;
use dictator_python::init_decree as create_python_plugin;
use dictator_ruby::init_decree as create_ruby_plugin;
use dictator_rust::init_decree as create_rust_plugin;
use dictator_supreme::init_decree as init_supreme_decree;
use dictator_typescript::init_decree as create_typescript_plugin;

use crate::files::FileTypes;

/// Initialize regime with all decrees for watch mode (all file types supported)
pub fn init_regime_for_watch(decree_config: Option<&DictateConfig>) -> Regime {
    let mut regime = Regime::new();

    // decree.supreme ALWAYS runs
    // Language-specific settings override supreme settings per file type
    if let Some(config) = decree_config
        && let Some(supreme_settings) = config.decree.get("supreme")
    {
        let supreme_config = dictator_supreme::config_from_decree_settings(supreme_settings);

        // Build language overrides: merge supreme + language settings
        let mut overrides = std::collections::HashMap::new();
        for lang in ["ruby", "typescript", "golang", "rust", "python"] {
            if let Some(lang_settings) = config.decree.get(lang) {
                let merged = dictator_supreme::merged_config(supreme_settings, lang_settings);
                overrides.insert(lang.to_string(), merged);
            }
        }

        regime.add_decree(dictator_supreme::init_decree_with_overrides(
            supreme_config,
            overrides,
        ));
    } else {
        regime.add_decree(init_supreme_decree());
    }

    // For watch mode, load all decrees (we don't know what files will change)
    if let Some(config) = decree_config
        && let Some(ruby_settings) = config.decree.get("ruby")
    {
        let ruby_config = dictator_ruby::config_from_decree_settings(ruby_settings);
        regime.add_decree(dictator_ruby::init_decree_with_config(ruby_config));
    } else {
        regime.add_decree(create_ruby_plugin());
    }

    if let Some(config) = decree_config
        && let Some(ts_settings) = config.decree.get("typescript")
    {
        let ts_config = dictator_typescript::config_from_decree_settings(ts_settings);
        regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
    } else {
        regime.add_decree(create_typescript_plugin());
    }

    regime.add_decree(create_golang_plugin());
    regime.add_decree(create_rust_plugin());
    regime.add_decree(create_python_plugin());

    if let Some(config) = decree_config
        && let Some(frontmatter_settings) = config.decree.get("frontmatter")
    {
        let frontmatter_config =
            dictator_frontmatter::config_from_decree_settings(frontmatter_settings);
        regime.add_decree(dictator_frontmatter::init_decree_with_config(
            frontmatter_config,
        ));
    } else {
        regime.add_decree(create_frontmatter_plugin());
    }

    regime
}

/// Initialize regime based on detected file types (for lint mode)
pub fn init_regime_for_files(
    file_types: &FileTypes,
    decree_config: Option<&DictateConfig>,
) -> Regime {
    let mut regime = Regime::new();

    // decree.supreme ALWAYS runs (on all files)
    // Language-specific settings override supreme settings per file type
    if let Some(config) = decree_config
        && let Some(supreme_settings) = config.decree.get("supreme")
    {
        let supreme_config = dictator_supreme::config_from_decree_settings(supreme_settings);

        // Build language overrides: merge supreme + language settings
        let mut overrides = std::collections::HashMap::new();
        for lang in ["ruby", "typescript", "golang", "rust", "python"] {
            if let Some(lang_settings) = config.decree.get(lang) {
                let merged = dictator_supreme::merged_config(supreme_settings, lang_settings);
                overrides.insert(lang.to_string(), merged);
            }
        }

        regime.add_decree(dictator_supreme::init_decree_with_overrides(
            supreme_config,
            overrides,
        ));
    } else {
        regime.add_decree(init_supreme_decree());
    }

    // Load language-specific decrees based on file types
    if file_types.has_ruby {
        if let Some(config) = decree_config
            && let Some(ruby_settings) = config.decree.get("ruby")
        {
            let ruby_config = dictator_ruby::config_from_decree_settings(ruby_settings);
            regime.add_decree(dictator_ruby::init_decree_with_config(ruby_config));
        } else {
            regime.add_decree(create_ruby_plugin());
        }
    }
    if file_types.has_typescript {
        if let Some(config) = decree_config
            && let Some(ts_settings) = config.decree.get("typescript")
        {
            let ts_config = dictator_typescript::config_from_decree_settings(ts_settings);
            regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
        } else {
            regime.add_decree(create_typescript_plugin());
        }
    }
    if file_types.has_golang {
        regime.add_decree(create_golang_plugin());
    }
    if file_types.has_rust {
        regime.add_decree(create_rust_plugin());
    }
    if file_types.has_python {
        regime.add_decree(create_python_plugin());
    }
    if file_types.has_configs {
        if let Some(config) = decree_config
            && let Some(frontmatter_settings) = config.decree.get("frontmatter")
        {
            let frontmatter_config =
                dictator_frontmatter::config_from_decree_settings(frontmatter_settings);
            regime.add_decree(dictator_frontmatter::init_decree_with_config(
                frontmatter_config,
            ));
        } else {
            regime.add_decree(create_frontmatter_plugin());
        }
    }

    regime
}
