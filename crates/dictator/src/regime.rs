//! Regime initialization and decree loading

use std::collections::HashMap;

use dictator_core::{DecreeSettings, DictateConfig, Regime};
use dictator_decree_abi::BoxDecree;
use dictator_supreme::SupremeConfig;

use crate::files::FileTypes;

/// A native language decree the regime can load: its config key plus the two
/// constructors (configured + built-in default).
struct LanguageDecree {
    /// `.dictate.toml` key, e.g. `"ruby"`.
    name: &'static str,
    /// Build the decree from explicit settings merged with supreme.
    with_configs: fn(&DecreeSettings, SupremeConfig) -> BoxDecree,
    /// Build the decree with built-in defaults (no config present).
    fallback: fn() -> BoxDecree,
}

/// The native language decrees, in load order. The order here must match the
/// `enabled` array in [`init_regime_for_files`].
fn language_decrees() -> [LanguageDecree; 5] {
    [
        LanguageDecree {
            name: "ruby",
            with_configs: |s, supreme| {
                dictator_ruby::init_decree_with_configs(
                    dictator_ruby::config_from_decree_settings(s),
                    supreme,
                )
            },
            fallback: dictator_ruby::init_decree,
        },
        LanguageDecree {
            name: "typescript",
            with_configs: |s, supreme| {
                dictator_typescript::init_decree_with_configs(
                    dictator_typescript::config_from_decree_settings(s),
                    supreme,
                )
            },
            fallback: dictator_typescript::init_decree,
        },
        LanguageDecree {
            name: "golang",
            with_configs: |s, supreme| {
                dictator_golang::init_decree_with_configs(
                    dictator_golang::config_from_decree_settings(s),
                    supreme,
                )
            },
            fallback: dictator_golang::init_decree,
        },
        LanguageDecree {
            name: "rust",
            with_configs: |s, supreme| {
                dictator_rust::init_decree_with_configs(
                    dictator_rust::config_from_decree_settings(s),
                    supreme,
                )
            },
            fallback: dictator_rust::init_decree,
        },
        LanguageDecree {
            name: "python",
            with_configs: |s, supreme| {
                dictator_python::init_decree_with_configs(
                    dictator_python::config_from_decree_settings(s),
                    supreme,
                )
            },
            fallback: dictator_python::init_decree,
        },
    ]
}

/// Check if a decree should be loaded based on config.
/// Returns true only if decree is configured and enabled != false
fn should_load_decree(config: Option<&DictateConfig>, key: &str) -> bool {
    config
        .and_then(|c| c.decree.get(key))
        .is_some_and(|s| s.enabled != Some(false))
}

/// Resolve the supreme config a language decree runs with: language settings
/// merged over `decree.supreme` when present, otherwise derived from the
/// language settings alone.
fn supreme_config_for(
    decree_config: Option<&DictateConfig>,
    settings: &DecreeSettings,
) -> SupremeConfig {
    decree_config
        .and_then(|c| c.decree.get("supreme"))
        .map_or_else(
            || dictator_supreme::config_from_decree_settings(settings),
            |base| dictator_supreme::merged_config(base, settings),
        )
}

/// Add the supreme decree, applying per-language overrides when configured.
///
/// decree.supreme runs as the default structural decree. When a language-specific
/// decree is enabled for a file type, it shadows supreme for that file type;
/// language settings override supreme settings via merged config.
pub(crate) fn add_supreme_decree(regime: &mut Regime, decree_config: Option<&DictateConfig>) {
    if let Some(config) = decree_config
        && let Some(supreme_settings) = config.decree.get("supreme")
    {
        let supreme_config = dictator_supreme::config_from_decree_settings(supreme_settings);

        // Build language overrides: merge supreme + language settings
        let mut overrides = HashMap::new();
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
        regime.add_decree(dictator_supreme::init_decree());
    }
}

/// Load a single language decree into the regime when `enabled` and configured.
fn load_language_decree(
    regime: &mut Regime,
    decree_config: Option<&DictateConfig>,
    decree: &LanguageDecree,
    enabled: bool,
) {
    if !(enabled && should_load_decree(decree_config, decree.name)) {
        return;
    }

    if let Some(config) = decree_config
        && let Some(settings) = config.decree.get(decree.name)
    {
        let supreme = supreme_config_for(decree_config, settings);
        regime.add_decree((decree.with_configs)(settings, supreme));
    } else {
        regime.add_decree((decree.fallback)());
    }
}

/// Load the frontmatter decree when `enabled` and configured.
fn load_frontmatter_decree(
    regime: &mut Regime,
    decree_config: Option<&DictateConfig>,
    enabled: bool,
) {
    if !(enabled && should_load_decree(decree_config, "frontmatter")) {
        return;
    }

    if let Some(config) = decree_config
        && let Some(settings) = config.decree.get("frontmatter")
    {
        let frontmatter_config = dictator_frontmatter::config_from_decree_settings(settings);
        regime.add_decree(dictator_frontmatter::init_decree_with_config(
            frontmatter_config,
        ));
    } else {
        regime.add_decree(dictator_frontmatter::init_decree());
    }
}

/// Initialize regime with all decrees for watch mode (all file types supported)
pub fn init_regime_for_watch(decree_config: Option<&DictateConfig>) -> Regime {
    let mut regime = Regime::new();
    regime.set_rule_ignores_from_config(decree_config);

    add_supreme_decree(&mut regime, decree_config);

    // For watch mode, load all decrees (we don't know what files will change)
    for decree in &language_decrees() {
        load_language_decree(&mut regime, decree_config, decree, true);
    }
    load_frontmatter_decree(&mut regime, decree_config, true);

    regime
}

/// Initialize regime based on detected file types (for lint mode)
pub fn init_regime_for_files(
    file_types: &FileTypes,
    decree_config: Option<&DictateConfig>,
) -> Regime {
    let mut regime = Regime::new();
    regime.set_rule_ignores_from_config(decree_config);

    add_supreme_decree(&mut regime, decree_config);

    // Load language-specific decrees based on file types. Order matches
    // `language_decrees()`.
    let enabled = [
        file_types.has_ruby,
        file_types.has_typescript,
        file_types.has_golang,
        file_types.has_rust,
        file_types.has_python,
    ];
    for (decree, &on) in language_decrees().iter().zip(enabled.iter()) {
        load_language_decree(&mut regime, decree_config, decree, on);
    }
    load_frontmatter_decree(&mut regime, decree_config, file_types.has_configs);

    regime
}
