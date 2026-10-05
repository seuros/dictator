use super::*;

#[test]
fn test_fix_trailing_whitespace() {
    let input = "hello world   \nfoo bar\t\t\n";
    let expected = "hello world\nfoo bar\n";
    assert_eq!(fix_structural_issues(input), expected);
}

#[test]
fn test_fix_crlf() {
    let input = "hello\r\nworld\r\n";
    let expected = "hello\nworld\n";
    assert_eq!(fix_structural_issues(input), expected);
}

#[test]
fn test_add_final_newline() {
    let input = "hello world";
    let expected = "hello world\n";
    assert_eq!(fix_structural_issues(input), expected);
}

#[test]
fn test_remove_extra_final_newlines() {
    let input = "hello world\n\n\n";
    let expected = "hello world\n";
    assert_eq!(fix_structural_issues(input), expected);
}

fn comment_space_diag(content: &str, line: usize) -> dictator_decree_abi::Diagnostic {
    lint_ruby(content)
        .into_iter()
        .filter(|d| d.rule == "ruby/comment-space")
        .nth(line)
        .expect("comment-space diagnostic")
}

fn lint_ruby(content: &str) -> dictator_decree_abi::Diagnostics {
    dictator_ruby::lint_source(content)
}

#[test]
fn fixes_ruby_comment_space() {
    let input = "#one\nsql = <<~SQL\n  #{table}\nSQL\n#two\n";
    let diag = comment_space_diag(input, 0);
    assert_eq!(
        apply_single_fix(input, &diag, None).as_deref(),
        Some("# one\nsql = <<~SQL\n  #{table}\nSQL\n# two\n")
    );
}

#[test]
fn scoped_ruby_comment_space_fix_touches_only_its_line() {
    let input = "#one\nx = 1\n#two\n";
    let diag = comment_space_diag(input, 1);
    assert_eq!(
        apply_fix_at_span(input, &diag, None).as_deref(),
        Some("#one\nx = 1\n# two\n")
    );
}

fn diag(rule: &str) -> dictator_decree_abi::Diagnostic {
    dictator_decree_abi::Diagnostic {
        rule: rule.to_string(),
        message: String::new(),
        enforced: true,
        span: dictator_decree_abi::Span::new(0, 0),
    }
}

#[test]
fn whitespace_fixes_keep_crlf_and_a_missing_final_newline() {
    let input = "a  \r\n  \r\nb\t";
    assert_eq!(
        apply_single_fix(input, &diag("supreme/trailing-whitespace"), None).as_deref(),
        Some("a\r\n\r\nb")
    );
    assert_eq!(
        apply_single_fix(input, &diag("supreme/blank-line-whitespace"), None).as_deref(),
        Some("a  \r\n\r\nb\t")
    );
}

#[test]
fn final_newline_matches_the_file_line_endings() {
    let fix = |input| apply_single_fix(input, &diag("supreme/missing-final-newline"), None);
    assert_eq!(fix("a\r\nb").as_deref(), Some("a\r\nb\r\n"));
    assert_eq!(fix("a\nb").as_deref(), Some("a\nb\n"));
    assert_eq!(fix("a\n"), None);
}

fn config(toml: &str) -> DictateConfig {
    toml::from_str(toml).expect("valid config")
}

#[test]
fn wants_crlf_resolves_owner_then_supreme() {
    let cfg = config(
        "[decree.supreme]\nline_endings = \"crlf\"\n\n[decree.golang]\nline_endings = \"lf\"\n",
    );
    assert!(wants_crlf(Some(&cfg), "supreme/wrong-line-ending"));
    assert!(
        wants_crlf(Some(&cfg), "ruby/mixed-line-endings"),
        "falls back to supreme"
    );
    assert!(
        !wants_crlf(Some(&cfg), "golang/mixed-line-endings"),
        "owner wins"
    );
    assert!(
        !wants_crlf(None, "supreme/mixed-line-endings"),
        "LF by default"
    );
}

#[test]
fn line_ending_fixes_convert_to_the_configured_ending() {
    let cfg = config("[decree.supreme]\nline_endings = \"crlf\"\n");
    let mixed = "a\r\nb\nc\n";
    assert_eq!(
        apply_single_fix(mixed, &diag("supreme/mixed-line-endings"), Some(&cfg)).as_deref(),
        Some("a\r\nb\r\nc\r\n")
    );
    assert_eq!(
        apply_single_fix("a\nb\n", &diag("supreme/wrong-line-ending"), Some(&cfg)).as_deref(),
        Some("a\r\nb\r\n")
    );
    assert_eq!(
        apply_single_fix(mixed, &diag("supreme/mixed-line-endings"), None).as_deref(),
        Some("a\nb\nc\n")
    );
}
