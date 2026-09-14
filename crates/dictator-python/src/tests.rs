use super::*;

#[test]
fn detects_file_too_long() {
    use std::fmt::Write;
    let mut src = String::new();
    for i in 0..400 {
        let _ = writeln!(src, "x = {i}");
    }
    let diags = lint_source(&src);
    assert!(
        diags.iter().any(|d| d.rule == "python/file-too-long"),
        "Should detect file with >380 code lines"
    );
}

#[test]
fn ignores_comments_in_line_count() {
    use std::fmt::Write;
    let mut src = String::new();
    for i in 0..380 {
        let _ = writeln!(src, "x = {i}");
    }
    for i in 0..60 {
        let _ = writeln!(src, "# Comment {i}");
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "python/file-too-long"),
        "Should not count comment-only lines"
    );
}

#[test]
fn ignores_blank_lines_in_count() {
    use std::fmt::Write;
    let mut src = String::new();
    for i in 0..380 {
        let _ = writeln!(src, "x = {i}");
    }
    for _ in 0..60 {
        src.push('\n');
    }
    let diags = lint_source(&src);
    assert!(
        !diags.iter().any(|d| d.rule == "python/file-too-long"),
        "Should not count blank lines"
    );
}

#[test]
fn detects_wrong_import_order_stdlib_after_third_party() {
    let src = r"
import requests
import os
import sys
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "python/import-order"),
        "Should detect stdlib import after third-party import"
    );
}

#[test]
fn detects_wrong_import_order_local_before_third_party() {
    let src = r"
from . import config
import requests
import os
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "python/import-order"),
        "Should detect wrong import order"
    );
}

#[test]
fn accepts_correct_import_order() {
    let src = r"
import os
import sys
import json
from typing import Dict, List
import requests
import django
from . import config
from .utils import helper
";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "python/import-order"),
        "Should accept correct import order"
    );
}

#[test]
fn detects_mixed_tabs_and_spaces() {
    let src = "def test():\n\tx = 1\n  y = 2\n";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "python/mixed-indentation"),
        "Should detect mixed tabs and spaces"
    );
}

#[test]
fn detects_inconsistent_indentation_depth() {
    let src = r"
def test():
  if True:
     x = 1
  y = 2
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "python/inconsistent-indentation"),
        "Should detect inconsistent indentation depth (3 spaces instead of 2 or 4)"
    );
}

#[test]
fn accepts_consistent_indentation() {
    let src = r"
def test():
    if True:
        x = 1
        y = 2
    z = 3
";
    let diags = lint_source(src);
    assert!(
        !diags
            .iter()
            .any(|d| d.rule == "python/mixed-indentation"
                || d.rule == "python/inconsistent-indentation"),
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
    let src = "# Comment 1\n# Comment 2\n# Comment 3\n";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "python/file-too-long"),
        "File with only comments should not trigger line count"
    );
}

#[test]
fn detects_stdlib_correctly() {
    assert!(is_python_stdlib("os"));
    assert!(is_python_stdlib("sys"));
    assert!(is_python_stdlib("json"));
    assert!(is_python_stdlib("typing"));
    assert!(is_python_stdlib("collections"));
    assert!(!is_python_stdlib("requests"));
    assert!(!is_python_stdlib("django"));
    assert!(!is_python_stdlib("numpy"));
}

#[test]
fn classifies_modules_correctly() {
    assert_eq!(classify_module("os"), ImportType::Stdlib);
    assert_eq!(classify_module("sys"), ImportType::Stdlib);
    assert_eq!(classify_module("json"), ImportType::Stdlib);
    assert_eq!(classify_module("requests"), ImportType::ThirdParty);
    assert_eq!(classify_module("django.conf"), ImportType::ThirdParty);
    assert_eq!(classify_module(".config"), ImportType::Local);
    assert_eq!(classify_module("..utils"), ImportType::Local);
}

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo():\n{long_comment}    pass\n");
    let config = PythonConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let python = Python::new(config, supreme);
    let diags = python.lint("test.py", &src);
    assert!(
        !diags.iter().any(|d| d.rule == "python/line-too-long"),
        "Should not flag long comment lines when ignore_comments is true"
    );
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo():\n{long_comment}    pass\n");
    let config = PythonConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let python = Python::new(config, supreme);
    let diags = python.lint("test.py", &src);
    assert!(
        diags.iter().any(|d| d.rule == "python/line-too-long"),
        "Should flag long comment lines when ignore_comments is false"
    );
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("    x = \"{}\"\n", "a".repeat(150));
    let src = format!("def foo():\n{long_code}    pass\n");
    let config = PythonConfig { ignore_comments: true, ..Default::default() };
    let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
    let python = Python::new(config, supreme);
    let diags = python.lint("test.py", &src);
    assert!(
        diags.iter().any(|d| d.rule == "python/line-too-long"),
        "Should still flag long code lines even when ignore_comments is true"
    );
}
