use super::*;

#[test]
fn detects_trailing_whitespace_and_tab() {
    let src = "def foo\n  bar \t\nend\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/trailing-whitespace"));
    assert!(diags.iter().any(|d| d.rule == "ruby/tab-character"));
}

#[test]
fn detects_missing_final_newline() {
    let src = "class Foo\nend";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/missing-final-newline"));
}

#[test]
fn enforces_comment_space() {
    let src = "#bad\n# good\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/comment-space"));
}

#[test]
fn comment_spacing_false_disables_comment_space_check() {
    let src = "#bad\n## Section\n";
    let config = RubyConfig { comment_spacing: false, ..Default::default() };
    let supreme = SupremeConfig::default();
    let diags = lint_source_with_configs(src, &config, &supreme);
    assert!(!diags.iter().any(|d| d.rule == "ruby/comment-space"));
}

#[test]
fn config_from_decree_settings_honors_comment_spacing() {
    let settings =
        dictator_core::DecreeSettings { comment_spacing: Some(false), ..Default::default() };
    let config = config_from_decree_settings(&settings);
    assert!(!config.comment_spacing);
}

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo\n{long_comment}end\n");
    let config = RubyConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(!diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo\n{long_comment}end\n");
    let config = RubyConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("  x = \"{}\"\n", "a".repeat(150));
    let src = format!("def foo\n{long_code}end\n");
    let config = RubyConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn detects_whitespace_only_blank_line() {
    let src = "def foo\n  bar\n    \nend\n"; // blank line has spaces
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/blank-line-whitespace"));
}
