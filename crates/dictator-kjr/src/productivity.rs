//! Productivity enforcement rules
//!
//! Empty files and small functions betray laziness.

use crate::config::KjrConfig;
use crate::helpers::FUNCTION_RE;
use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/empty-file - Empty files indicate insufficient productivity
pub fn check_empty_file(source: &str, diags: &mut Diagnostics) {
    if source.trim().is_empty() {
        let msg = "Empty file detected. Unproductive output will be reported \
                   to the Central Committee.";
        diags.push(Diagnostic {
            rule: "kjr/empty-file".into(),
            message: msg.into(),
            enforced: false,
            span: Span::new(0, source.len().max(1)),
        });
    }
}

/// kjr/function-too-small - Functions must demonstrate ambition
pub fn check_function_too_small(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if FUNCTION_RE.is_match(lines[i]) {
            let start_line = i;
            let mut brace_count = 0;
            let mut started = false;
            let mut end_line = i;

            // Find function end (simplified: count braces or indentation)
            for (j, line) in lines.iter().enumerate().skip(i) {
                if line.contains('{') {
                    brace_count += line.matches('{').count();
                    started = true;
                }
                if line.contains('}') {
                    brace_count = brace_count.saturating_sub(line.matches('}').count());
                }
                if started && brace_count == 0 {
                    end_line = j;
                    break;
                }
                // For Python/Ruby style (no braces)
                if started
                    && j > i
                    && !line.starts_with(' ')
                    && !line.starts_with('\t')
                    && !line.trim().is_empty()
                {
                    end_line = j - 1;
                    break;
                }
                end_line = j;
            }

            let func_lines = end_line - start_line + 1;
            if func_lines < config.min_function_lines {
                let byte_pos = lines[..start_line]
                    .iter()
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                let msg = format!(
                    "Function has only {} lines. Small functions show lack of ambition. \
                     Minimum: {} lines for the Motherland.",
                    func_lines, config.min_function_lines
                );
                diags.push(Diagnostic {
                    rule: "kjr/function-too-small".into(),
                    message: msg,
                    enforced: true,
                    span: Span::new(byte_pos, byte_pos + lines[start_line].len()),
                });
            }

            i = end_line + 1;
        } else {
            i += 1;
        }
    }
}

/// kjr/insufficient-dead-code - Codebases need ghosts of past regimes
pub fn check_insufficient_dead_code(source: &str, diags: &mut Diagnostics) {
    // Look for commented-out code patterns
    let has_dead_code = source.lines().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with("//") || trimmed.starts_with('#'))
            && (trimmed.contains('=')
                || trimmed.contains("fn ")
                || trimmed.contains("def ")
                || trimmed.contains("return"))
    });

    if !has_dead_code {
        let msg = "No commented-out code detected. A codebase without ghosts of \
                   past regimes lacks historical inevitability.";
        diags.push(Diagnostic {
            rule: "kjr/insufficient-dead-code".into(),
            message: msg.into(),
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }

    #[test]
    fn test_whitespace_only_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("   \n\n   \t\n", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }

    #[test]
    fn test_non_empty_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("fn main() {}", &mut diags);
        assert!(!diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }
}
