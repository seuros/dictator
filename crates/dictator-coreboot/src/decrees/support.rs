use dictator_decree_abi::{Decree, Diagnostic, Diagnostics, Span};

use crate::CorebootDecree;

pub(crate) fn push_diag(
    decree: &CorebootDecree,
    diags: &mut Diagnostics,
    rule: &str,
    message: String,
    line_offset: usize,
    start_col: usize,
    end_col: usize,
) {
    let start = line_offset + start_col;
    let end = line_offset + end_col.max(start_col + 1);

    diags.push(Diagnostic {
        rule: decree.rule(rule),
        message,
        span: Span::new(start, end),
        enforced: true,
    });
}

/// Blanks out comments and string/char literals while preserving column
/// alignment, so spacing and keyword checks never trip on prose.
pub(crate) fn sanitize_code_line(line: &str, in_block_comment: &mut bool) -> String {
    let bytes = line.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0usize;

    while i < bytes.len() {
        if *in_block_comment {
            out[i] = b' ';
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                out[i + 1] = b' ';
                *in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            for ch in out.iter_mut().skip(i) {
                *ch = b' ';
            }
            break;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            out[i] = b' ';
            out[i + 1] = b' ';
            *in_block_comment = true;
            i += 2;
            continue;
        }

        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            out[i] = b' ';
            i += 1;

            while i < bytes.len() {
                out[i] = b' ';
                if i + 1 < bytes.len() && bytes[i] == b'\\' {
                    out[i + 1] = b' ';
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        i += 1;
    }

    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

/// Check for a word (not part of a larger identifier) followed by `(`.
pub(crate) fn find_bare_call(line: &str, name: &str) -> Option<usize> {
    let pattern = format!("{name}(");
    let mut start = 0usize;

    while let Some(pos) = line[start..].find(&pattern) {
        let idx = start + pos;
        let bytes = line.as_bytes();
        let prev_ok =
            idx == 0 || !(bytes[idx - 1].is_ascii_alphanumeric() || bytes[idx - 1] == b'_');
        if prev_ok {
            return Some(idx);
        }
        start = idx + 1;
    }

    None
}
