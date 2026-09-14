//! Linter output parsing - converts external linter JSON to Dictator Diagnostics
//!
//! Supports: `RuboCop`, Ruff, `ESLint`, Biome, Clippy
//! Each parser extracts fixability info to set `enforced` dynamically.

use dictator_decree_abi::{Diagnostic, Span};
use serde::Deserialize;

macro_rules! parse_or_default {
    ($parser:ident, $json:expr) => {
        $parser($json).unwrap_or_default()
    };
}

/// Parse linter output based on command name
#[must_use]
pub fn parse_linter_output(command: &str, json: &str) -> Vec<Diagnostic> {
    match command {
        "rubocop" => parse_or_default!(parse_rubocop, json),
        "ruff" => parse_or_default!(parse_ruff, json),
        "eslint" => parse_or_default!(parse_eslint, json),
        "biome" => parse_or_default!(parse_biome, json),
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
// Biome - uses `tags` array containing "fixable"
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeOutput {
    diagnostics: Vec<BiomeDiagnostic>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeDiagnostic {
    category: Option<String>,
    description: Option<String>,
    message: Option<BiomeMessage>,
    location: Option<BiomeLocation>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeMessage {
    // Message is an array of markup elements, we extract content
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeLocation {
    path: Option<BiomeResource>,
    span: Option<(usize, usize)>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BiomeResource {
    file: Option<String>,
}

fn parse_biome(json: &str) -> Result<Vec<Diagnostic>, serde_json::Error> {
    let output: BiomeOutput = serde_json::from_str(json)?;
    let mut diagnostics = Vec::new();

    for diag in output.diagnostics {
        // Category looks like "lint/style/useConst" -> "biome/style/useConst"
        let rule = diag.category.map_or_else(
            || "biome/unknown".to_string(),
            |c| {
                // Strip "lint/" prefix if present, keep the rest
                let stripped = c.strip_prefix("lint/").unwrap_or(&c);
                format!("biome/{stripped}")
            },
        );

        // Get file path from location
        let file_path = diag
            .location
            .as_ref()
            .and_then(|l| l.path.as_ref())
            .and_then(|p| p.file.as_ref())
            .map_or_else(|| "unknown".to_string(), String::clone);

        // Get span for line info (span is [start, end] byte offsets)
        let span_info = diag
            .location
            .as_ref()
            .and_then(|l| l.span)
            .map_or_else(String::new, |(start, _)| format!(":{start}"));

        // Use description or extract from message
        let message_text = diag
            .description
            .or_else(|| diag.message.and_then(|m| m.content))
            .unwrap_or_default();

        // Check if "fixable" tag is present
        let enforced = diag
            .tags
            .as_ref()
            .is_some_and(|tags| tags.iter().any(|t| t == "fixable"));

        diagnostics.push(Diagnostic {
            rule,
            message: format!("[{file_path}{span_info}] {message_text}"),
            enforced,
            span: Span::new(0, 0),
        });
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
mod tests;
