use super::*;

#[test]
fn test_parse_rubocop_correctable() {
    let json = r#"{
            "files": [{
                "path": "test.rb",
                "offenses": [{
                    "message": "Trailing whitespace detected.",
                    "cop_name": "Layout/TrailingWhitespace",
                    "correctable": true,
                    "location": {"line": 1, "column": 10}
                }]
            }]
        }"#;
    let diags = parse_rubocop(json).unwrap();
    assert_eq!(diags.len(), 1);
    assert!(diags[0].rule.contains("TrailingWhitespace"));
    assert!(diags[0].enforced); // correctable = true
}

#[test]
fn test_parse_rubocop_not_correctable() {
    let json = r#"{
            "files": [{
                "path": "test.rb",
                "offenses": [{
                    "message": "Method too long.",
                    "cop_name": "Metrics/MethodLength",
                    "correctable": false,
                    "location": {"line": 1, "column": 1}
                }]
            }]
        }"#;
    let diags = parse_rubocop(json).unwrap();
    assert!(!diags[0].enforced); // correctable = false
}

#[test]
fn test_parse_ruff_safe_fix() {
    let json = r#"[{
            "filename": "test.py",
            "code": "F401",
            "message": "`os` imported but unused",
            "fix": {"applicability": "safe"},
            "location": {"row": 1, "column": 8}
        }]"#;
    let diags = parse_ruff(json).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "ruff/F401");
    assert!(diags[0].enforced); // safe fix
}

#[test]
fn test_parse_ruff_unsafe_fix() {
    let json = r#"[{
            "filename": "test.py",
            "code": "E501",
            "message": "line too long",
            "fix": {"applicability": "unsafe"},
            "location": {"row": 1, "column": 1}
        }]"#;
    let diags = parse_ruff(json).unwrap();
    assert!(!diags[0].enforced); // unsafe fix
}

#[test]
fn test_parse_ruff_no_fix() {
    let json = r#"[{
            "filename": "test.py",
            "code": "E999",
            "message": "syntax error",
            "location": {"row": 1, "column": 1}
        }]"#;
    let diags = parse_ruff(json).unwrap();
    assert!(!diags[0].enforced); // no fix available
}

#[test]
fn test_parse_eslint_with_fix() {
    let json = r#"[{
            "filePath": "test.js",
            "messages": [{
                "ruleId": "semi",
                "message": "Missing semicolon.",
                "line": 1,
                "column": 5,
                "fix": {"range": [4, 4], "text": ";"}
            }]
        }]"#;
    let diags = parse_eslint(json).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "eslint/semi");
    assert!(diags[0].enforced); // has fix
}

#[test]
fn test_parse_eslint_without_fix() {
    let json = r#"[{
            "filePath": "test.js",
            "messages": [{
                "ruleId": "no-unused-vars",
                "message": "'x' is defined but never used.",
                "line": 1,
                "column": 5
            }]
        }]"#;
    let diags = parse_eslint(json).unwrap();
    assert!(!diags[0].enforced); // no fix
}

#[test]
fn test_parse_clippy_machine_applicable() {
    // Clippy outputs one JSON object per line - must be single line
    let json = concat!(
        r#"{"reason":"compiler-message","message":{"#,
        r#""code":{"code":"clippy::needless_return"},"#,
        r#""message":"unneeded `return`","#,
        r#""spans":[{"file_name":"src/lib.rs","line_start":13,"#,
        r#""column_start":12,"is_primary":true}],"#,
        r#""children":[{"suggestion_applicability":"MachineApplicable"}]}}"#
    );
    let diags = parse_clippy(json);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].enforced); // MachineApplicable
}

#[test]
fn test_parse_clippy_not_applicable() {
    // Clippy outputs one JSON object per line - must be single line
    let json = concat!(
        r#"{"reason":"compiler-message","message":{"#,
        r#""code":{"code":"clippy::must_use_candidate"},"#,
        r#""message":"this method could have a `#[must_use]` attribute","#,
        r#""spans":[{"file_name":"src/lib.rs","line_start":13,"#,
        r#""column_start":12,"is_primary":true}],"#,
        r#""children":[{"suggestion_applicability":"MaybeIncorrect"}]}}"#
    );
    let diags = parse_clippy(json);
    assert!(!diags[0].enforced); // MaybeIncorrect != MachineApplicable
}

#[test]
fn test_parse_biome_with_fixable() {
    let json = r#"{
            "diagnostics": [{
                "category": "lint/style/useConst",
                "description": "This let declares a variable that is only assigned once.",
                "location": {
                    "path": {"file": "test.js"},
                    "span": [10, 20]
                },
                "tags": ["fixable"]
            }]
        }"#;
    let diags = parse_biome(json).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "biome/style/useConst");
    assert!(diags[0].enforced); // has fixable tag
}

#[test]
fn test_parse_biome_without_fixable() {
    let json = r#"{
            "diagnostics": [{
                "category": "lint/correctness/noUnusedVariables",
                "description": "This variable is unused.",
                "location": {
                    "path": {"file": "test.ts"},
                    "span": [5, 15]
                },
                "tags": []
            }]
        }"#;
    let diags = parse_biome(json).unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule, "biome/correctness/noUnusedVariables");
    assert!(!diags[0].enforced); // no fixable tag
}

#[test]
fn test_parse_biome_empty_diagnostics() {
    let json = r#"{"diagnostics": []}"#;
    let diags = parse_biome(json).unwrap();
    assert!(diags.is_empty());
}
