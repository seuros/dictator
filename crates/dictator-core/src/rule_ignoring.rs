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
mod tests {
    use super::*;
    use crate::config::DecreeSettings;
    use dictator_decree_abi::Span;

    #[test]
    fn ignores_by_filename() {
        let mut settings = DecreeSettings::default();
        settings.ignore.insert(
            "tab-character".to_string(),
            RuleIgnore { filenames: vec!["Makefile".to_string()], extensions: vec![] },
        );
        let mut config = DictateConfig::default();
        config.decree.insert("supreme".to_string(), settings);

        let ignores = build_rule_ignores(Some(&config));

        let diag = Diagnostic {
            rule: "supreme/tab-character".to_string(),
            message: "tab found".to_string(),
            span: Span::new(0, 0),
            enforced: false,
        };

        assert!(is_rule_ignored_for_path(&ignores, Utf8Path::new("Makefile"), &diag));
        assert!(!is_rule_ignored_for_path(&ignores, Utf8Path::new("other.txt"), &diag));
    }

    #[test]
    fn ignores_by_extension() {
        let mut settings = DecreeSettings::default();
        settings.ignore.insert(
            "tab-character".to_string(),
            RuleIgnore { filenames: vec![], extensions: vec!["md".to_string(), "MDX".to_string()] },
        );
        let mut config = DictateConfig::default();
        config.decree.insert("supreme".to_string(), settings);

        let ignores = build_rule_ignores(Some(&config));

        let diag = Diagnostic {
            rule: "supreme/tab-character".to_string(),
            message: "tab found".to_string(),
            span: Span::new(0, 0),
            enforced: false,
        };

        assert!(is_rule_ignored_for_path(&ignores, Utf8Path::new("README.md"), &diag));
        assert!(is_rule_ignored_for_path(&ignores, Utf8Path::new("doc.mdx"), &diag));
        assert!(!is_rule_ignored_for_path(&ignores, Utf8Path::new("code.rs"), &diag));
    }

    #[test]
    fn does_not_ignore_unconfigured_rules() {
        let mut settings = DecreeSettings::default();
        settings.ignore.insert(
            "tab-character".to_string(),
            RuleIgnore { filenames: vec!["Makefile".to_string()], extensions: vec![] },
        );
        let mut config = DictateConfig::default();
        config.decree.insert("supreme".to_string(), settings);

        let ignores = build_rule_ignores(Some(&config));

        let diag = Diagnostic {
            rule: "supreme/trailing-whitespace".to_string(),
            message: "whitespace found".to_string(),
            span: Span::new(0, 0),
            enforced: false,
        };

        // Different rule, should not be ignored
        assert!(!is_rule_ignored_for_path(&ignores, Utf8Path::new("Makefile"), &diag));
    }
}
