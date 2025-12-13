//! Regime initialization and stalint checking.

use camino::Utf8Path;
use dictator_core::{Regime, Source};
use serde_json::Value;
use std::collections::HashMap;

use super::utils::{collect_files, make_snippet};

/// Run stalint check and return violations
pub fn run_stalint_check(paths: &[String]) -> Vec<Value> {
    let regime = init_regime_from_config();
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut violations = Vec::new();

    for path in paths {
        let path = std::path::Path::new(path);
        if !path.exists() {
            continue;
        }

        let files = collect_files(path);
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };

            // Use relative path if within cwd (saves tokens)
            let relative = file.strip_prefix(&cwd).unwrap_or(&file);
            let path_str = relative.to_str().unwrap_or("<invalid>");
            let source = Source {
                path: Utf8Path::new(path_str),
                text: &text,
            };

            if let Ok(diags) = regime.enforce(&[source]) {
                for diag in &diags {
                    let snippet = make_snippet(&text, &diag.span, 160);
                    violations.push(serde_json::json!({
                        "file": path_str,
                        "rule": diag.rule,
                        "message": diag.message,
                        "enforced": diag.enforced,
                        "snippet": snippet,
                    }));
                }
            }
        }
    }

    violations
}

/// Initialize regime with configured decrees
pub fn init_regime_from_config() -> Regime {
    let mut regime = Regime::new();

    let config = dictator_core::DictateConfig::load_default();
    regime.set_rule_ignores_from_config(config.as_ref());

    // Load decree configuration and apply to supreme plugin
    // Language-specific settings override supreme settings per file type
    if let Some(config) = config
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

        // Load native decrees declared in config with overrides
        load_native_decrees(&mut regime, &config.decree, supreme_settings);
    } else {
        regime.add_decree(dictator_supreme::init_decree());
    }

    regime
}

/// Load native decrees based on config
fn load_native_decrees(
    regime: &mut Regime,
    decree_config: &HashMap<String, dictator_core::config::DecreeSettings>,
    supreme_settings: &dictator_core::config::DecreeSettings,
) {
    for (decree_name, settings) in decree_config {
        match decree_name.as_str() {
            "typescript" => {
                let ts_config = dictator_typescript::config_from_decree_settings(settings);
                let ts_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                regime.add_decree(dictator_typescript::init_decree_with_configs(
                    ts_config, ts_supreme,
                ));
            }
            "python" => {
                let py_config = dictator_python::config_from_decree_settings(settings);
                let py_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                regime.add_decree(dictator_python::init_decree_with_configs(
                    py_config, py_supreme,
                ));
            }
            "golang" => {
                let go_config = dictator_golang::config_from_decree_settings(settings);
                let go_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                regime.add_decree(dictator_golang::init_decree_with_configs(
                    go_config, go_supreme,
                ));
            }
            "rust" => {
                let rs_config = dictator_rust::config_from_decree_settings(settings);
                let rs_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                regime.add_decree(dictator_rust::init_decree_with_configs(
                    rs_config, rs_supreme,
                ));
            }
            "ruby" => {
                let rb_config = dictator_ruby::config_from_decree_settings(settings);
                let rb_supreme = dictator_supreme::merged_config(supreme_settings, settings);
                regime.add_decree(dictator_ruby::init_decree_with_configs(
                    rb_config, rb_supreme,
                ));
            }
            "frontmatter" => {
                let fm_config = dictator_frontmatter::config_from_decree_settings(settings);
                regime.add_decree(dictator_frontmatter::init_decree_with_config(fm_config));
            }
            _ => {} // Already loaded above; custom WASM decrees handled elsewhere
        }
    }
}
