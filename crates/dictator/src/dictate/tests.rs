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
