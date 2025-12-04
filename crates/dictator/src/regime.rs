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

/// Check if a decree should be loaded based on config.
/// Returns true if: no config, no decree entry, or enabled != false
fn should_load_decree(config: Option<&DictateConfig>, key: &str) -> bool {
    config
        .and_then(|c| c.decree.get(key))
        .is_none_or(|s| s.enabled != Some(false))
}

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
    if should_load_decree(decree_config, "ruby") {
        if let Some(config) = decree_config
            && let Some(ruby_settings) = config.decree.get("ruby")
        {
            let ruby_config = dictator_ruby::config_from_decree_settings(ruby_settings);
            regime.add_decree(dictator_ruby::init_decree_with_config(ruby_config));
        } else {
            regime.add_decree(create_ruby_plugin());
        }
    }

    if should_load_decree(decree_config, "typescript") {
        if let Some(config) = decree_config
            && let Some(ts_settings) = config.decree.get("typescript")
        {
            let ts_config = dictator_typescript::config_from_decree_settings(ts_settings);
            regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
        } else {
            regime.add_decree(create_typescript_plugin());
        }
    }

    if should_load_decree(decree_config, "golang") {
        if let Some(config) = decree_config
            && let Some(golang_settings) = config.decree.get("golang")
        {
            let golang_config = dictator_golang::config_from_decree_settings(golang_settings);
            regime.add_decree(dictator_golang::init_decree_with_config(golang_config));
        } else {
            regime.add_decree(create_golang_plugin());
        }
    }
    if should_load_decree(decree_config, "rust") {
        if let Some(config) = decree_config
            && let Some(rust_settings) = config.decree.get("rust")
        {
            let rust_config = dictator_rust::config_from_decree_settings(rust_settings);
            regime.add_decree(dictator_rust::init_decree_with_config(rust_config));
        } else {
            regime.add_decree(create_rust_plugin());
        }
    }
    if should_load_decree(decree_config, "python") {
        if let Some(config) = decree_config
            && let Some(python_settings) = config.decree.get("python")
        {
            let python_config = dictator_python::config_from_decree_settings(python_settings);
            regime.add_decree(dictator_python::init_decree_with_config(python_config));
        } else {
            regime.add_decree(create_python_plugin());
        }
    }

    if should_load_decree(decree_config, "frontmatter") {
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
    if file_types.has_ruby && should_load_decree(decree_config, "ruby") {
        if let Some(config) = decree_config
            && let Some(ruby_settings) = config.decree.get("ruby")
        {
            let ruby_config = dictator_ruby::config_from_decree_settings(ruby_settings);
            regime.add_decree(dictator_ruby::init_decree_with_config(ruby_config));
        } else {
            regime.add_decree(create_ruby_plugin());
        }
    }
    if file_types.has_typescript && should_load_decree(decree_config, "typescript") {
        if let Some(config) = decree_config
            && let Some(ts_settings) = config.decree.get("typescript")
        {
            let ts_config = dictator_typescript::config_from_decree_settings(ts_settings);
            regime.add_decree(dictator_typescript::init_decree_with_config(ts_config));
        } else {
            regime.add_decree(create_typescript_plugin());
        }
    }
    if file_types.has_golang && should_load_decree(decree_config, "golang") {
        if let Some(config) = decree_config
            && let Some(golang_settings) = config.decree.get("golang")
        {
            let golang_config = dictator_golang::config_from_decree_settings(golang_settings);
            regime.add_decree(dictator_golang::init_decree_with_config(golang_config));
        } else {
            regime.add_decree(create_golang_plugin());
        }
    }
    if file_types.has_rust && should_load_decree(decree_config, "rust") {
        if let Some(config) = decree_config
            && let Some(rust_settings) = config.decree.get("rust")
        {
            let rust_config = dictator_rust::config_from_decree_settings(rust_settings);
            regime.add_decree(dictator_rust::init_decree_with_config(rust_config));
        } else {
            regime.add_decree(create_rust_plugin());
        }
    }
    if file_types.has_python && should_load_decree(decree_config, "python") {
        if let Some(config) = decree_config
            && let Some(python_settings) = config.decree.get("python")
        {
            let python_config = dictator_python::config_from_decree_settings(python_settings);
            regime.add_decree(dictator_python::init_decree_with_config(python_config));
        } else {
            regime.add_decree(create_python_plugin());
        }
    }
    if file_types.has_configs && should_load_decree(decree_config, "frontmatter") {
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
