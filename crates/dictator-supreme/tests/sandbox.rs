//! Integration tests over the shared `sandbox/` violation corpus.
//! These pin the byte-exact fixtures documented in `sandbox/EXPECTED_VIOLATIONS.md`
//! so that editors, autocrlf, or `dictator dictate` can't silently defang them.

use dictator_supreme::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

fn rules(rel: &str) -> Vec<String> {
    lint_source(&sandbox(rel)).into_iter().map(|d| d.rule).collect()
}

#[test]
fn trailing_whitespace_fixtures() {
    for rel in [
        "golang/trailing_whitespace.go",
        "python/trailing_whitespace.py",
        "rust/01_trailing_whitespace.rs",
        "typescript/BadReactComponent.tsx",
    ] {
        assert!(
            rules(rel).iter().any(|r| r == "supreme/trailing-whitespace"),
            "{rel} should trigger supreme/trailing-whitespace"
        );
    }
}

#[test]
fn missing_final_newline_fixtures() {
    for rel in [
        "golang/missing_newline.go",
        "python/no_final_newline.py",
        "rust/03_missing_final_newline.rs",
        "typescript/TooLongFile.ts",
        "typescript/BadReactComponent.tsx",
        "typescript/UtilityFunctions.ts",
        "typescript/InconsistentIndentation.ts",
    ] {
        assert!(
            rules(rel).iter().any(|r| r == "supreme/missing-final-newline"),
            "{rel} should trigger supreme/missing-final-newline"
        );
    }
}

#[test]
fn mixed_line_ending_fixtures() {
    for rel in [
        "golang/mixed_line_endings.go",
        "python/long_file_mixed_endings.py",
        "typescript/TooLongFile.ts",
    ] {
        assert!(
            rules(rel).iter().any(|r| r == "supreme/mixed-line-endings"),
            "{rel} should trigger supreme/mixed-line-endings (CRLF bytes were normalized away?)"
        );
    }
}
