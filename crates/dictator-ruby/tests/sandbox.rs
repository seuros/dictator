//! Integration tests over `sandbox/ruby/` (see `sandbox/EXPECTED_VIOLATIONS.md`).
//! Ruby wraps the supreme rules with the `ruby/` owner prefix.

use dictator_ruby::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/ruby/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

fn rules(rel: &str) -> Vec<String> {
    lint_source(&sandbox(rel))
        .into_iter()
        .map(|d| d.rule)
        .collect()
}

#[test]
fn trailing_whitespace_fixture() {
    assert!(
        rules("trailing_whitespace.rb")
            .iter()
            .any(|r| r == "ruby/trailing-whitespace"),
        "trailing_whitespace.rb should trigger ruby/trailing-whitespace"
    );
}

#[test]
fn mixed_indentation_fixture() {
    assert!(
        rules("mixed_indentation.rb")
            .iter()
            .any(|r| r == "ruby/tab-character"),
        "mixed_indentation.rb uses tabs and should trigger ruby/tab-character"
    );
}

#[test]
fn no_final_newline_fixture() {
    assert!(
        rules("no_final_newline.rb")
            .iter()
            .any(|r| r == "ruby/missing-final-newline"),
        "no_final_newline.rb should trigger ruby/missing-final-newline"
    );
}

#[test]
fn mixed_line_endings_fixture() {
    assert!(
        rules("mixed_line_endings.rb")
            .iter()
            .any(|r| r == "ruby/mixed-line-endings"),
        "mixed_line_endings.rb should trigger ruby/mixed-line-endings (CRLF bytes were normalized away?)"
    );
}

#[test]
fn too_long_file_fixture() {
    // 316 code lines, limit 300 (blanks/comments excluded).
    assert!(
        rules("too_long_file.rb")
            .iter()
            .any(|r| r == "ruby/file-too-long"),
        "too_long_file.rb should trigger ruby/file-too-long"
    );
}

#[test]
fn wrong_comment_spacing_fixture() {
    let count = rules("wrong_comment_spacing.rb")
        .iter()
        .filter(|r| r.as_str() == "ruby/comment-space")
        .count();
    assert_eq!(
        count, 13,
        "wrong_comment_spacing.rb has 13 `#comment` instances (shebang and magic comments exempt)"
    );
}
