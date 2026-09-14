use super::*;

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("fn main() {{\n{long_comment}}}\n");
    let config = RustConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(!diags.iter().any(|d| d.rule == "rust/line-too-long"));
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("fn main() {{\n{long_comment}}}\n");
    let config = RustConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "rust/line-too-long"));
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("    let x = \"{}\";\n", "a".repeat(150));
    let src = format!("fn main() {{\n{long_code}}}\n");
    let config = RustConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "rust/line-too-long"));
}
