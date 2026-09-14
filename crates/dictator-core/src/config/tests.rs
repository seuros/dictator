use super::*;

#[test]
fn parses_valid_config() {
    let toml = r#"
[decree.supreme]
trailing_whitespace = "deny"
tabs_vs_spaces = "spaces"
tab_width = 2
final_newline = "require"
line_endings = "lf"
max_line_length = 120
blank_line_whitespace = "deny"

[decree.supreme.ignore.tab-character]
filenames = ["Makefile"]
extensions = ["md", "mdx"]

[decree.ruby]
max_lines = 300
ignore_comments = true
ignore_blank_lines = true
method_visibility_order = ["public", "protected", "private"]
comment_spacing = true

[decree.typescript]
max_lines = 350
ignore_comments = true
ignore_blank_lines = true
import_order = ["system", "external", "internal"]
"#;

    let config: DictateConfig = toml::from_str(toml).unwrap();

    // Validate all decrees
    for (name, settings) in &config.decree {
        settings.validate().unwrap_or_else(|e| {
            panic!("decree.{name} validation failed: {e}");
        });
    }

    assert!(config.decree.contains_key("supreme"));
    assert!(config.decree.contains_key("ruby"));
    assert!(config.decree.contains_key("typescript"));

    let supreme = config.decree.get("supreme").unwrap();
    assert_eq!(supreme.max_line_length, Some(120));
    assert_eq!(supreme.tabs_vs_spaces, Some("spaces".to_string()));
    assert!(supreme.ignore.contains_key("tab-character"));
    let ignore = supreme.ignore.get("tab-character").unwrap();
    assert_eq!(ignore.filenames, vec!["Makefile".to_string()]);
    assert_eq!(ignore.extensions, vec!["md".to_string(), "mdx".to_string()]);

    let ruby = config.decree.get("ruby").unwrap();
    assert_eq!(ruby.max_lines, Some(300));
    assert_eq!(ruby.ignore_comments, Some(true));

    let ts = config.decree.get("typescript").unwrap();
    assert_eq!(ts.max_lines, Some(350));
}

#[test]
fn rejects_invalid_max_line_length() {
    let settings = DecreeSettings {
        max_line_length: Some(10), // Too small (min 40)
        ..Default::default()
    };

    let result = settings.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("40-500"));
}

#[test]
fn rejects_negative_max_line_length_at_parse() {
    // Negative values fail at TOML parse for usize
    let toml = r"
[decree.supreme]
max_line_length = -340
";
    let result: Result<DictateConfig, _> = toml::from_str(toml);
    assert!(result.is_err());
}

#[test]
fn rejects_invalid_tabs_vs_spaces() {
    let settings = DecreeSettings {
        tabs_vs_spaces: Some("tab".to_string()), // Should be "tabs"
        ..Default::default()
    };

    let result = settings.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("tabs"));
}

#[test]
fn rejects_invalid_line_endings() {
    let settings = DecreeSettings {
        line_endings: Some("windows".to_string()), // Should be "crlf"
        ..Default::default()
    };

    let result = settings.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("lf"));
}

#[test]
fn rejects_tab_width_out_of_range() {
    let settings = DecreeSettings {
        tab_width: Some(32), // Max is 16
        ..Default::default()
    };

    let result = settings.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("1-16"));
}

#[test]
fn rejects_max_lines_out_of_range() {
    let settings = DecreeSettings {
        max_lines: Some(10), // Min is 50
        ..Default::default()
    };

    let result = settings.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("50-5000"));
}

#[test]
fn accepts_valid_settings() {
    let settings = DecreeSettings {
        trailing_whitespace: Some("deny".to_string()),
        tabs_vs_spaces: Some("spaces".to_string()),
        tab_width: Some(4),
        final_newline: Some("require".to_string()),
        line_endings: Some("lf".to_string()),
        max_line_length: Some(100),
        blank_line_whitespace: Some("allow".to_string()),
        max_lines: Some(500),
        ..Default::default()
    };

    assert!(settings.validate().is_ok());
}

#[test]
fn accepts_none_values() {
    let settings = DecreeSettings::default();
    assert!(settings.validate().is_ok());
}

#[test]
fn profile_inheritance_applies_parent_before_child() {
    let toml = r#"
[decree.supreme]
max_line_length = 100

[profile.relaxed.decree.supreme]
max_line_length = 120

[profile.ci]
inherits = "relaxed"

[profile.ci.decree.supreme]
max_line_length = 80
"#;

    let config: DictateConfig = toml::from_str(toml).unwrap();
    let effective = config.get_profile_config("ci").unwrap();

    assert_eq!(effective.decree["supreme"].max_line_length, Some(80));
}

#[test]
fn profile_inheritance_rejects_missing_parent() {
    let toml = r#"
[profile.ci]
inherits = "missing"
"#;

    let config: DictateConfig = toml::from_str(toml).unwrap();
    let err = config.get_profile_config("ci").unwrap_err();

    assert!(err.contains("missing"));
}

#[test]
fn from_file_validates_profile_decree_settings() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(
        &mut file,
        br#"
[profile.ci.decree.supreme]
max_line_length = 10
"#,
    )
    .unwrap();

    let err = DictateConfig::from_file(file.path()).unwrap_err();

    assert!(err.to_string().contains("profile.ci.decree.supreme"));
    assert!(err.to_string().contains("40-500"));
}
