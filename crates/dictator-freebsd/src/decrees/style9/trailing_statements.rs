use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_trailing_statements(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;
    let mut in_block_comment = false;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let clean = sanitize_code_line(line, &mut in_block_comment);
        let trimmed = clean.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            offset += raw.len();
            continue;
        }

        // if/while/for (...) must not carry a statement on the same line.
        for kw in ["if", "while", "for"] {
            if let Some((kw_pos, _open, close)) = find_keyword_condition_end(&clean, kw) {
                if kw == "while" && clean[..kw_pos].trim_end().ends_with('}') {
                    continue;
                }
                if has_trailing_statement_after_cond(&clean, close) {
                    push_diag(
                        decree,
                        diags,
                        "trailing-statement",
                        "trailing statements should be on next line".to_string(),
                        offset,
                        close,
                        close + 1,
                        true,
                    );
                }
                break;
            }
        }

        // else should only be followed by nothing, '{', or if.
        if let Some(pos) = clean.find("else") {
            let prev = clean[..pos].trim_end();
            let boundary_ok = pos == 0 || !is_ident_byte(clean.as_bytes()[pos - 1]);
            if boundary_ok && (prev.is_empty() || prev.ends_with('}')) {
                let mut after = clean[pos + "else".len()..].trim_start();
                if !after.starts_with("if") {
                    if let Some(rest) = after.strip_prefix('{') {
                        after = rest.trim_start();
                    }
                    if let Some(rest) = after.strip_prefix('\\') {
                        after = rest.trim_start();
                    }
                    if !after.is_empty() {
                        push_diag(
                            decree,
                            diags,
                            "trailing-statement",
                            "trailing statements should be on next line".to_string(),
                            offset,
                            pos,
                            pos + "else".len(),
                            true,
                        );
                    }
                }
            }
        }

        // `} if (...)` should be split.
        if let Some(pos) = clean.find("} if") {
            push_diag(
                decree,
                diags,
                "trailing-statement",
                "trailing statements should be on next line".to_string(),
                offset,
                pos + 2,
                pos + 4,
                true,
            );
        }

        // case/default labels should not have trailing general statements.
        if trimmed.starts_with("case ") || trimmed.starts_with("default:") {
            if let Some(colon_idx) = clean.find(':') {
                let mut after = clean[colon_idx + 1..].trim_start();
                let ok_return = after.starts_with("return ");
                let ok_chain = after.starts_with("case ") || after.starts_with("default:");
                if !ok_return {
                    if let Some(rest) = after.strip_prefix('{') {
                        after = rest.trim_start();
                    }
                    if let Some(rest) = after.strip_prefix('\\') {
                        after = rest.trim_start();
                    }
                    if !after.is_empty() && !ok_chain {
                        push_diag(
                            decree,
                            diags,
                            "trailing-statement",
                            "trailing statements should be on next line".to_string(),
                            offset,
                            colon_idx,
                            colon_idx + 1,
                            true,
                        );
                    }
                }
            }
        }

        offset += raw.len();
    }
}
