use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_block_comment_style(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;
    let mut in_block = false;
    let mut prev_star_col: Option<usize> = None;
    let mut need_star_after_open = false;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);

        if !in_block {
            if let Some(open) = line.find("/*") {
                if let Some(close_rel) = line[open + 2..].find("*/") {
                    let close = open + 2 + close_rel;
                    let suffix = &line[close + 2..];
                    let suffix_ok = suffix
                        .chars()
                        .all(|c| c.is_ascii_whitespace() || c == ')' || c == '}');
                    if !suffix_ok {
                        push_diag(
                            decree,
                            diags,
                            "block-comment-leading",
                            "Block comments use a leading /* on a separate line".to_string(),
                            offset,
                            open,
                            open + 2,
                            true,
                        );
                    }
                } else {
                    let after = line[open + 2..].trim_start();
                    if !after.is_empty() && !after.starts_with('*') && !after.starts_with('-') {
                        push_diag(
                            decree,
                            diags,
                            "block-comment-leading",
                            "Block comments use a leading /* on a separate line".to_string(),
                            offset,
                            open,
                            open + 2,
                            true,
                        );
                    }
                    prev_star_col = Some(expanded_len(&line[..open + 1]));
                    in_block = true;
                    need_star_after_open = true;
                }
            }
        } else {
            if need_star_after_open {
                if line.trim_start().starts_with('*') {
                    need_star_after_open = false;
                } else {
                    push_diag(
                        decree,
                        diags,
                        "block-comment-middle",
                        "Block comments use * on subsequent lines".to_string(),
                        offset,
                        0,
                        1,
                        true,
                    );
                    need_star_after_open = false;
                }
            }

            if let Some(star_idx) = line.find('*') {
                if line[..star_idx].trim().is_empty() {
                    let star_col = expanded_len(&line[..star_idx]);
                    if let Some(prev_col) = prev_star_col
                        && prev_col != star_col
                    {
                        push_diag(
                            decree,
                            diags,
                            "block-comment-align",
                            "Block comments should align the * on each line".to_string(),
                            offset,
                            0,
                            1,
                            false,
                        );
                    }
                    prev_star_col = Some(star_col);
                }
            }

            if let Some(close) = line.find("*/") {
                let close_line = line.trim();
                if close_line != "*/" && !close_line.ends_with("**/") {
                    push_diag(
                        decree,
                        diags,
                        "block-comment-trailing",
                        "Block comments use a trailing */ on a separate line".to_string(),
                        offset,
                        close,
                        close + 2,
                        true,
                    );
                }
                in_block = false;
                prev_star_col = None;
                need_star_after_open = false;
            }
        }

        offset += raw.len();
    }
}
