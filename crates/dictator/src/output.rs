//! Output formatting and diagnostic display

use dictator_decree_abi::{Diagnostic, Span};
use serde::Serialize;

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
