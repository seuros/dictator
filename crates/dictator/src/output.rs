//! Output formatting and diagnostic display

use dictator_decree_abi::{Diagnostic, Span};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize)]
pub struct SerializableDiagnostic {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub rule: String,
    pub message: String,
    pub enforced: bool,
    pub span: Span,
}

pub fn print_diagnostic(path: &str, source: &str, diag: &Diagnostic) {
    let (line, col) = byte_to_line_col(source, diag.span.start);
    let status = if diag.enforced { "🔧" } else { "❌" };
    println!(
        "{path}:{line}:{col}: {status} {rule}: {msg}",
        rule = diag.rule,
        msg = diag.message
    );
}

/// Print one persona-voiced summary line per decree that flagged violations
/// in this file, e.g. "Beastie is not happy with foo.8 (3 violations)".
pub fn print_persona_summary(
    path: &str,
    diags: &[&Diagnostic],
    personas: &HashMap<String, String>,
) {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for diag in diags {
        let decree_name = diag.rule.split('/').next().unwrap_or(diag.rule.as_str());
        *counts.entry(decree_name).or_insert(0) += 1;
    }

    for (decree_name, count) in counts {
        let persona = personas
            .get(decree_name)
            .map_or("The Dictator", String::as_str);
        let noun = if count == 1 {
            "violation"
        } else {
            "violations"
        };
        println!("{persona} is not happy with {path} ({count} {noun})");
    }
}

pub fn byte_to_line_col(src: &str, byte_idx: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i == byte_idx {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
