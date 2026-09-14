//! Rule ignore evaluation.

use camino::Utf8Path;
use dictator_decree_abi::Diagnostic;
use std::collections::HashMap;

use crate::config::{DictateConfig, RuleIgnore};

/// Type alias for rule ignores: decree_name -> rule_name -> RuleIgnore
pub type RuleIgnores = HashMap<String, HashMap<String, RuleIgnore>>;

/// Build rule ignores from a loaded `.dictate.toml`.
///
/// Ignores are keyed by decree name (`decree.<name>`) and rule name (the
/// portion after `{decree}/` in diagnostic rule identifiers).
pub fn build_rule_ignores(config: Option<&DictateConfig>) -> RuleIgnores {
    let mut ignores = RuleIgnores::new();

    let Some(cfg) = config else {
        return ignores;
    };

    for (decree_name, settings) in &cfg.decree {
        if settings.ignore.is_empty() {
            continue;
        }
        ignores.insert(decree_name.clone(), settings.ignore.clone());
    }

    ignores
}

/// Check if a diagnostic should be ignored for the given path.
pub fn is_rule_ignored_for_path(
    rule_ignores: &RuleIgnores,
    path: &Utf8Path,
    diag: &Diagnostic,
) -> bool {
    if rule_ignores.is_empty() {
        return false;
    }

    let Some((decree, rule_name)) = diag.rule.split_once('/') else {
        return false;
    };

    let Some(rules) = rule_ignores.get(decree) else {
        return false;
    };
    let Some(ignore) = rules.get(rule_name) else {
        return false;
    };

    let filename = path.file_name().unwrap_or("");
    if ignore.filenames.iter().any(|f| f == filename) {
        return true;
    }

    let Some(ext) = path.extension() else {
        return false;
    };
    ignore.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests;
