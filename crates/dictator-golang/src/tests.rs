use super::*;

#[test]
fn detects_file_too_long() {
    use std::fmt::Write;
    // Create a file with 460 code lines (excluding blank lines and comments)
    let mut src = String::new();
    for i in 0..460 {
        let _ = writeln!(src, "x{i} := {i}");
    }
    let diags = lint_source(&src);
    assert!(
        diags.iter().any(|d| d.rule == "golang/file-too-long"),
        "Should detect file with >450 code lines"
    );
}

#[test]
fn ignores_comments_in_line_count() {
    use std::fmt::Write;
    // 440 code lines + 60 comment lines = 500 total, but only 440 counted
    let mut src = String::new();
    for i in 0..440 {
        let _ = writeln!(src, "x{i} := {i}");
    }
    for i in 0..60 {
        let _ = writeln!(src, "// Comment {i}");
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "golang/file-too-long"),
        "Should not count comment-only lines"
    );
}

#[test]
fn ignores_blank_lines_in_count() {
    use std::fmt::Write;
    // 440 code lines + 60 blank lines = 500 total, but only 440 counted
    let mut src = String::new();
    for i in 0..440 {
        let _ = writeln!(src, "x{i} := {i}");
    }
    for _ in 0..60 {
        src.push('\n');
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "golang/file-too-long"),
        "Should not count blank lines"
    );
}

#[test]
fn detects_spaces_instead_of_tabs() {
    let src = "package main\n\nfunc test() {\n    x := 1\n}\n";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should detect spaces used for indentation"
    );
}

#[test]
fn allows_tabs_for_indentation() {
    let src = "package main\n\nfunc test() {\n\tx := 1\n}\n";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should allow tabs for indentation"
    );
}

#[test]
fn detects_spaces_at_line_start() {
    let src = "package main\n\nfunc test() {\n    \tx := 1\n}\n";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should detect spaces at start of indented line"
    );
}

#[test]
fn handles_empty_file() {
    let src = "";
    let diags = lint_source(src);
    assert!(diags.is_empty(), "Empty file should have no violations");
}

#[test]
fn handles_file_with_only_comments() {
    let src = "// Comment 1\n// Comment 2\n/* Block comment */\n";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "golang/file-too-long"),
        "File with only comments should not trigger line count"
    );
}

#[test]
fn allows_blank_lines() {
    let src = "package main\n\n\nfunc test() {\n\tx := 1\n}\n";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should allow blank lines"
    );
}

#[test]
fn allows_spaces_inside_raw_string_literals() {
    // Simulates Cobra command help text with intentional space indentation
    let src = concat!(
        "package main\n\nvar cmd = &cobra.Command{\n",
        "\tUse:   \"test\",\n",
        "\tExample: `\n    test foo bar\n    test baz qux`,\n}\n"
    );
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should allow spaces inside raw string literals (backtick strings)"
    );
}

#[test]
fn allows_spaces_in_multiline_raw_string() {
    let src = "package main\n\nvar help = `\n  Usage:\n    command [flags]\n`\n";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should allow spaces in multiline raw string"
    );
}

#[test]
fn detects_spaces_after_raw_string_closes() {
    let src = "package main\n\nvar x = `raw`\n  y := 1\n";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should detect spaces after raw string closes"
    );
}

#[test]
fn handles_multiple_raw_strings() {
    let src = "package main\n\nvar a = `\n  first\n`\nvar b = `\n  second\n`\n";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Should handle multiple raw strings correctly"
    );
}

#[test]
fn handles_raw_string_with_backticks_inline() {
    // Two backticks on same line = opens and closes
    let src = "package main\n\nvar x = `inline`\n  y := 1\n";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "golang/spaces-instead-of-tabs"),
        "Inline raw strings should not affect next line"
    );
}

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("package main\n{long_comment}func main() {{}}\n");
    let config = GolangConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        !diags.iter().any(|d| d.rule == "golang/line-too-long"),
        "Should not flag long comment lines when ignore_comments is true"
    );
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("package main\n{long_comment}func main() {{}}\n");
    let config = GolangConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        diags.iter().any(|d| d.rule == "golang/line-too-long"),
        "Should flag long comment lines when ignore_comments is false"
    );
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("\tx := \"{}\"\n", "a".repeat(150));
    let src = format!("package main\n{long_code}func main() {{}}\n");
    let config = GolangConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        diags.iter().any(|d| d.rule == "golang/line-too-long"),
        "Should still flag long code lines even when ignore_comments is true"
    );
}
