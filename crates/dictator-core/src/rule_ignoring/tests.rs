use super::*;
use crate::config::DecreeSettings;
use dictator_decree_abi::Span;

#[test]
fn ignores_by_filename() {
    let mut settings = DecreeSettings::default();
    settings.ignore.insert(
        "tab-character".to_string(),
        RuleIgnore {
            filenames: vec!["Makefile".to_string()],
            extensions: vec![],
        },
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

    assert!(is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("Makefile"),
        &diag
    ));
    assert!(!is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("other.txt"),
        &diag
    ));
}

#[test]
fn ignores_by_extension() {
    let mut settings = DecreeSettings::default();
    settings.ignore.insert(
        "tab-character".to_string(),
        RuleIgnore {
            filenames: vec![],
            extensions: vec!["md".to_string(), "MDX".to_string()],
        },
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

    assert!(is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("README.md"),
        &diag
    ));
    assert!(is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("doc.mdx"),
        &diag
    ));
    assert!(!is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("code.rs"),
        &diag
    ));
}

#[test]
fn does_not_ignore_unconfigured_rules() {
    let mut settings = DecreeSettings::default();
    settings.ignore.insert(
        "tab-character".to_string(),
        RuleIgnore {
            filenames: vec!["Makefile".to_string()],
            extensions: vec![],
        },
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
    assert!(!is_rule_ignored_for_path(
        &ignores,
        Utf8Path::new("Makefile"),
        &diag
    ));
}
