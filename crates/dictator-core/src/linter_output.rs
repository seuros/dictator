//! Linter output parsing - converts external linter JSON to Dictator Diagnostics
//!
//! Supports: `RuboCop`, Ruff, `ESLint`, Clippy
//! Each parser extracts fixability info to set `enforced` dynamically.

use dictator_decree_abi::{Diagnostic, Span};
use serde::Deserialize;

/// Parse linter output based on command name
#[must_use]
pub fn parse_linter_output(command: &str, json: &str) -> Vec<Diagnostic> {
    match command {
        "rubocop" => parse_rubocop(json).unwrap_or_default(),
        "ruff" => parse_ruff(json).unwrap_or_default(),
        "eslint" => parse_eslint(json).unwrap_or_default(),
        "clippy" | "cargo-clippy" => parse_clippy(json),
        _ => vec![],
    }
}

// ============================================================================
// RuboCop - uses `correctable` field
// ============================================================================

#[derive(Debug, Deserialize)]
struct RubocopOutput {
    files: Vec<RubocopFile>,
}

#[derive(Debug, Deserialize)]
struct RubocopFile {
    path: String,
    offenses: Vec<RubocopOffense>,
}

#[derive(Debug, Deserialize)]
struct RubocopOffense {
    message: String,
    cop_name: String,
    correctable: Option<bool>,
    location: RubocopLocation,
}

#[derive(Debug, Deserialize)]
struct RubocopLocation {
    line: usize,
    column: usize,
}

fn parse_rubocop(json: &str) -> Result<Vec<Diagnostic>, serde_json::Error> {
    let output: RubocopOutput = serde_json::from_str(json)?;
    let mut diagnostics = Vec::new();

    for file in output.files {
        for offense in file.offenses {
            diagnostics.push(Diagnostic {
                rule: format!("rubocop/{}", offense.cop_name),
                message: format!(
                    "[{}:{}:{}] {}",
                    file.path, offense.location.line, offense.location.column, offense.message
                ),
                enforced: offense.correctable.unwrap_or(false),
                span: Span::new(0, 0),
            });
        }
    }

    Ok(diagnostics)
}

// ============================================================================
// Ruff - uses `fix` object presence
// ============================================================================

#[derive(Debug, Deserialize)]
struct RuffDiagnostic {
    filename: String,
    code: String,
    message: String,
    fix: Option<RuffFix>,
    location: RuffLocation,
}

#[derive(Debug, Deserialize)]
struct RuffFix {
    applicability: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuffLocation {
    row: usize,
    column: usize,
}

fn parse_ruff(json: &str) -> Result<Vec<Diagnostic>, serde_json::Error> {
    let output: Vec<RuffDiagnostic> = serde_json::from_str(json)?;
    let mut diagnostics = Vec::new();

    for diag in output {
        // Ruff fix applicability: "safe", "unsafe", or "display-only"
        let enforced = diag
            .fix
            .as_ref()
            .is_some_and(|f| f.applicability.as_deref() == Some("safe"));

        diagnostics.push(Diagnostic {
            rule: format!("ruff/{}", diag.code),
            message: format!(
                "[{}:{}:{}] {}",
                diag.filename, diag.location.row, diag.location.column, diag.message
            ),
            enforced,
            span: Span::new(0, 0),
        });
    }

    Ok(diagnostics)
}

// ============================================================================
// ESLint - uses `fix` object presence
// ============================================================================

#[derive(Debug, Deserialize)]
struct EslintFile {
    #[serde(rename = "filePath")]
    file_path: String,
    messages: Vec<EslintMessage>,
}

#[derive(Debug, Deserialize)]
struct EslintMessage {
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    fix: Option<EslintFix>,
}

#[derive(Debug, Deserialize)]
struct EslintFix {
    // range and text exist but we only care about presence
}

fn parse_eslint(json: &str) -> Result<Vec<Diagnostic>, serde_json::Error> {
    let output: Vec<EslintFile> = serde_json::from_str(json)?;
    let mut diagnostics = Vec::new();

    for file in output {
        for msg in file.messages {
            let rule = msg.rule_id.map_or_else(
                || "eslint/parse-error".to_string(),
                |r| format!("eslint/{r}"),
            );

            diagnostics.push(Diagnostic {
                rule,
                message: format!(
                    "[{}:{}:{}] {}",
                    file.file_path,
                    msg.line.unwrap_or(0),
                    msg.column.unwrap_or(0),
                    msg.message
                ),
                enforced: msg.fix.is_some(),
                span: Span::new(0, 0),
            });
        }
    }

    Ok(diagnostics)
}

// ============================================================================
// Clippy - uses `children[].suggestion_applicability`
// ============================================================================

#[derive(Debug, Deserialize)]
struct ClippyMessage {
    reason: Option<String>,
    message: Option<ClippyDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct ClippyDiagnostic {
    code: Option<ClippyCode>,
    message: String,
    spans: Vec<ClippySpan>,
    children: Option<Vec<ClippyChild>>,
}

#[derive(Debug, Deserialize)]
struct ClippyCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct ClippySpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
    is_primary: bool,
}

#[derive(Debug, Deserialize)]
struct ClippyChild {
    suggestion_applicability: Option<String>,
}

fn parse_clippy(json: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Clippy outputs one JSON object per line
    for line in json.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(msg) = serde_json::from_str::<ClippyMessage>(line)
            && msg.reason.as_deref() == Some("compiler-message")
            && let Some(diag) = msg.message
        {
            let rule = diag.code.map_or_else(
                || "clippy/unknown".to_string(),
                |c| format!("clippy/{}", c.code),
            );

            // Get primary span for location
            let location = diag
                .spans
                .iter()
                .find(|s| s.is_primary)
                .or_else(|| diag.spans.first())
                .map_or_else(String::new, |s| {
                    format!("[{}:{}:{}] ", s.file_name, s.line_start, s.column_start)
                });

            // MachineApplicable = safe to auto-fix
            let enforced = diag.children.as_ref().is_some_and(|children| {
                children
                    .iter()
                    .any(|c| c.suggestion_applicability.as_deref() == Some("MachineApplicable"))
            });

            diagnostics.push(Diagnostic {
                rule,
                message: format!("{}{}", location, diag.message),
                enforced,
                span: Span::new(0, 0),
            });
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rubocop_correctable() {
        let json = r#"{"files":[{"path":"test.rb","offenses":[{"message":"Trailing whitespace detected.","cop_name":"Layout/TrailingWhitespace","correctable":true,"location":{"line":1,"column":10}}]}]}"#;
        let diags = parse_rubocop(json).unwrap();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].rule.contains("TrailingWhitespace"));
        assert!(diags[0].enforced); // correctable = true
    }

    #[test]
    fn test_parse_rubocop_not_correctable() {
        let json = r#"{"files":[{"path":"test.rb","offenses":[{"message":"Method too long.","cop_name":"Metrics/MethodLength","correctable":false,"location":{"line":1,"column":1}}]}]}"#;
        let diags = parse_rubocop(json).unwrap();
        assert!(!diags[0].enforced); // correctable = false
    }

    #[test]
    fn test_parse_ruff_safe_fix() {
        let json = r#"[{"filename":"test.py","code":"F401","message":"`os` imported but unused","fix":{"applicability":"safe"},"location":{"row":1,"column":8}}]"#;
        let diags = parse_ruff(json).unwrap();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "ruff/F401");
        assert!(diags[0].enforced); // safe fix
    }

    #[test]
    fn test_parse_ruff_unsafe_fix() {
        let json = r#"[{"filename":"test.py","code":"E501","message":"line too long","fix":{"applicability":"unsafe"},"location":{"row":1,"column":1}}]"#;
        let diags = parse_ruff(json).unwrap();
        assert!(!diags[0].enforced); // unsafe fix
    }

    #[test]
    fn test_parse_ruff_no_fix() {
        let json = r#"[{"filename":"test.py","code":"E999","message":"syntax error","location":{"row":1,"column":1}}]"#;
        let diags = parse_ruff(json).unwrap();
        assert!(!diags[0].enforced); // no fix available
    }

    #[test]
    fn test_parse_eslint_with_fix() {
        let json = r#"[{"filePath":"test.js","messages":[{"ruleId":"semi","message":"Missing semicolon.","line":1,"column":5,"fix":{"range":[4,4],"text":";"}}]}]"#;
        let diags = parse_eslint(json).unwrap();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "eslint/semi");
        assert!(diags[0].enforced); // has fix
    }

    #[test]
    fn test_parse_eslint_without_fix() {
        let json = r#"[{"filePath":"test.js","messages":[{"ruleId":"no-unused-vars","message":"'x' is defined but never used.","line":1,"column":5}]}]"#;
        let diags = parse_eslint(json).unwrap();
        assert!(!diags[0].enforced); // no fix
    }

    #[test]
    fn test_parse_clippy_machine_applicable() {
        let json = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::needless_return"},"message":"unneeded `return`","spans":[{"file_name":"src/lib.rs","line_start":13,"column_start":12,"is_primary":true}],"children":[{"suggestion_applicability":"MachineApplicable"}]}}"#;
        let diags = parse_clippy(json);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].enforced); // MachineApplicable
    }

    #[test]
    fn test_parse_clippy_not_applicable() {
        let json = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::must_use_candidate"},"message":"this method could have a `#[must_use]` attribute","spans":[{"file_name":"src/lib.rs","line_start":13,"column_start":12,"is_primary":true}],"children":[{"suggestion_applicability":"MaybeIncorrect"}]}}"#;
        let diags = parse_clippy(json);
        assert!(!diags[0].enforced); // MaybeIncorrect != MachineApplicable
    }
}
