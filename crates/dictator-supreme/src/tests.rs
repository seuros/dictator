use super::*;

#[test]
fn detects_trailing_whitespace() {
    let src = "hello world  \n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "supreme/trailing-whitespace"));
}

#[test]
fn detects_tabs_when_spaces_expected() {
    let src = "hello\tworld\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "supreme/tab-character"));
}

#[test]
fn allows_tabs_when_configured() {
    let src = "\thello world\n";
    let config = SupremeConfig { tabs_vs_spaces: TabsOrSpaces::Tabs, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(!diags.iter().any(|d| d.rule == "supreme/tab-character"));
}

#[test]
fn detects_spaces_when_tabs_expected() {
    let src = "  hello world\n";
    let config = SupremeConfig { tabs_vs_spaces: TabsOrSpaces::Tabs, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
}

#[test]
fn detects_single_space_when_tabs_expected() {
    let src = " hello world\n";
    let config = SupremeConfig { tabs_vs_spaces: TabsOrSpaces::Tabs, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
}

#[test]
fn detects_mixed_tabs_and_spaces_when_tabs_expected() {
    let src = "\t hello world\n"; // tab then space
    let config = SupremeConfig { tabs_vs_spaces: TabsOrSpaces::Tabs, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/space-indentation"));
}

#[test]
fn detects_missing_final_newline() {
    let src = "hello world";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "supreme/missing-final-newline"));
}

#[test]
fn allows_missing_final_newline_when_configured() {
    let src = "hello world";
    let config = SupremeConfig { final_newline: false, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(!diags.iter().any(|d| d.rule == "supreme/missing-final-newline"));
}

#[test]
fn detects_blank_line_whitespace() {
    let src = "line1\n   \nline2\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "supreme/blank-line-whitespace"));
}

#[test]
fn detects_line_too_long() {
    let src = format!("{}\n", "x".repeat(150));
    let config = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_config(&src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/line-too-long"));
}

#[test]
fn skips_line_length_when_disabled() {
    let src = format!("{}\n", "x".repeat(500));
    let diags = lint_source(&src); // Default has max_line_length: None
    assert!(!diags.iter().any(|d| d.rule == "supreme/line-too-long"));
}

#[test]
fn detects_mixed_line_endings() {
    let src = "line1\r\nline2\nline3\r\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "supreme/mixed-line-endings"));
}

#[test]
fn detects_crlf_when_lf_expected() {
    let src = "line1\r\nline2\r\n";
    let config = SupremeConfig { line_endings: LineEnding::Lf, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/wrong-line-ending"));
}

#[test]
fn detects_lf_when_crlf_expected() {
    let src = "line1\nline2\n";
    let config = SupremeConfig { line_endings: LineEnding::Crlf, ..Default::default() };
    let diags = lint_source_with_config(src, &config);
    assert!(diags.iter().any(|d| d.rule == "supreme/wrong-line-ending"));
}

#[test]
fn handles_empty_file() {
    let src = "";
    let diags = lint_source(src);
    // Empty file is valid (has no violations except maybe missing final newline)
    assert!(diags.is_empty() || diags.len() == 1);
}

#[test]
fn handles_single_line_with_newline() {
    let src = "hello world\n";
    let diags = lint_source(src);
    assert!(diags.is_empty());
}
