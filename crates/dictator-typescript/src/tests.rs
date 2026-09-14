use super::*;

#[test]
fn detects_file_too_long() {
    use std::fmt::Write;
    // Create a file with 400 code lines (excluding blank lines and comments)
    let mut src = String::new();
    for i in 0..400 {
        let _ = writeln!(src, "const x{i} = {i};");
    }
    let diags = lint_source(&src);
    assert!(
        diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "Should detect file with >350 code lines"
    );
}

#[test]
fn ignores_comments_in_line_count() {
    use std::fmt::Write;
    // 340 code lines + 60 comment lines = 400 total, but only 340 counted
    let mut src = String::new();
    for i in 0..340 {
        let _ = writeln!(src, "const x{i} = {i};");
    }
    for i in 0..60 {
        let _ = writeln!(src, "// Comment {i}");
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "Should not count comment-only lines"
    );
}

#[test]
fn ignores_blank_lines_in_count() {
    use std::fmt::Write;
    // 340 code lines + 60 blank lines = 400 total, but only 340 counted
    let mut src = String::new();
    for i in 0..340 {
        let _ = writeln!(src, "const x{i} = {i};");
    }
    for _ in 0..60 {
        src.push('\n');
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "Should not count blank lines"
    );
}

#[test]
fn detects_wrong_import_order_system_after_external() {
    let src = r"
import { format } from 'date-fns';
import * as fs from 'fs';
import { config } from './config';
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "typescript/import-order"),
        "Should detect system import after external import"
    );
}

#[test]
fn detects_wrong_import_order_internal_before_external() {
    let src = r"
import { config } from './config';
import { format } from 'date-fns';
import * as fs from 'fs';
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "typescript/import-order"),
        "Should detect wrong import order"
    );
}

#[test]
fn accepts_correct_import_order() {
    let src = r"
import * as fs from 'fs';
import * as path from 'path';
import { format } from 'date-fns';
import axios from 'axios';
import { config } from './config';
import type { Logger } from './types';
";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "typescript/import-order"),
        "Should accept correct import order"
    );
}

#[test]
fn detects_mixed_tabs_and_spaces() {
    let src = "function test() {\n\tconst x = 1;\n  const y = 2;\n}\n";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "typescript/mixed-indentation"),
        "Should detect mixed tabs and spaces"
    );
}

#[test]
fn detects_inconsistent_indentation_depth() {
    let src = r"
function test() {
  if (true) {
     const x = 1;
  }
}
";
    let diags = lint_source(src);
    assert!(
        diags
            .iter()
            .any(|d| d.rule == "typescript/inconsistent-indentation"),
        "Should detect inconsistent indentation depth (3 spaces instead of 2 or 4)"
    );
}

#[test]
fn accepts_consistent_indentation() {
    let src = r"
function test() {
  if (true) {
    const x = 1;
    const y = 2;
  }
}
";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "typescript/mixed-indentation"
                || d.rule == "typescript/inconsistent-indentation"),
        "Should accept consistent indentation"
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
        !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
        "File with only comments should not trigger line count"
    );
}

#[test]
fn detects_nodejs_builtins_correctly() {
    assert!(is_nodejs_builtin("fs"));
    assert!(is_nodejs_builtin("path"));
    assert!(is_nodejs_builtin("crypto"));
    assert!(is_nodejs_builtin("events"));
    assert!(!is_nodejs_builtin("date-fns"));
    assert!(!is_nodejs_builtin("lodash"));
    assert!(!is_nodejs_builtin("./config"));
}

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("function foo() {{\n{long_comment}}}\n");
    let config = TypeScriptConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        !diags.iter().any(|d| d.rule == "typescript/line-too-long"),
        "Should ignore long comment lines when ignore_comments is true"
    );
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("// {}\n", "x".repeat(150));
    let src = format!("function foo() {{\n{long_comment}}}\n");
    let config = TypeScriptConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        diags.iter().any(|d| d.rule == "typescript/line-too-long"),
        "Should detect long comment lines when ignore_comments is false"
    );
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("  const x = \"{}\";\n", "a".repeat(150));
    let src = format!("function foo() {{\n{long_code}}}\n");
    let config = TypeScriptConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(
        diags.iter().any(|d| d.rule == "typescript/line-too-long"),
        "Should still detect long code lines even with ignore_comments enabled"
    );
}
