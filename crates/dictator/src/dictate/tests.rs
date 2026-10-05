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
        apply_single_fix(input, &diag).as_deref(),
        Some("# one\nsql = <<~SQL\n  #{table}\nSQL\n# two\n")
    );
}

#[test]
fn scoped_ruby_comment_space_fix_touches_only_its_line() {
    let input = "#one\nx = 1\n#two\n";
    let diag = comment_space_diag(input, 1);
    assert_eq!(
        apply_fix_at_span(input, &diag).as_deref(),
        Some("#one\nx = 1\n# two\n")
    );
}
