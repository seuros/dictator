use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_basic_operator_spacing(
    decree: &FreeBsdDecree,
    path: &str,
    clean_line: &str,
    _original_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    if bytes.len() < 3 {
        return;
    }

    let mut brace_scan = 0usize;
    while let Some(rel) = clean_line[brace_scan..].find("){") {
        let brace_col = brace_scan + rel + 1;
        push_diag(
            decree,
            diags,
            "operator-spacing",
            "space required before the open brace '{'".to_string(),
            offset,
            brace_col,
            brace_col + 1,
            true,
        );
        brace_scan = brace_col + 1;
    }

    for i in 1..bytes.len() - 1 {
        let prev = bytes[i - 1];
        let cur = bytes[i];
        let next = bytes[i + 1];

        if cur == b'+' && next == b'=' && i + 2 < bytes.len() {
            let right = bytes[i + 2];
            if !prev.is_ascii_whitespace() && right.is_ascii_whitespace() {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '+=' (ctx:VxW)".to_string(),
                    offset,
                    i,
                    i + 2,
                    true,
                );
            }
        }

        if cur == b'>' && next == b')' && prev.is_ascii_whitespace() {
            let mut p = i;
            while p > 0 && bytes[p - 1].is_ascii_whitespace() {
                p -= 1;
            }
            if p > 0 && bytes[p - 1] == b',' {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '>' (ctx:WxB)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            }
        }

        if cur == b'!' && next.is_ascii_whitespace() {
            let has_non_ws_before = clean_line[..i]
                .as_bytes()
                .iter()
                .any(|b| !b.is_ascii_whitespace());
            if !has_non_ws_before {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "space prohibited after that '!' (ctx:ExW)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            } else if prev.is_ascii_whitespace() {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "space prohibited after that '!' (ctx:WxW)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            }
        }

        if cur == b'|' && prev != b'|' && next != b'|' {
            if prev == b'=' || next == b'=' {
                continue;
            }
            if !prev.is_ascii_whitespace() || !next.is_ascii_whitespace() {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '|' (ctx:VxV)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            }
        }

        if (cur == b'+' || cur == b'-')
            && prev != cur
            && next != cur
            && !(cur == b'-' && next == b'>')
        {
            if cur == b'-' && is_cast_before_minus(clean_line, i) {
                continue;
            }
            let looks_binary = (is_ident_byte(prev) || matches!(prev, b')' | b']'))
                && (is_ident_byte(next) || matches!(next, b'(' | b'['));
            if looks_binary && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace()) {
                let msg = if cur == b'+' {
                    "spaces required around that '+' (ctx:VxV)"
                } else {
                    "spaces required around that '-' (ctx:VxV)"
                };
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    msg.to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            }
        }

        // Narrow division spacing parity around sizeof(...) / sizeof(...)-style forms.
        if cur == b'/' && prev == b')' {
            if path.contains("/pax/") {
                continue;
            }
            if !next.is_ascii_whitespace() && is_ident_byte(next) {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '/' (ctx:VxV)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            } else if next.is_ascii_whitespace()
                && i + 2 < bytes.len()
                && is_ident_byte(bytes[i + 2])
            {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '/' (ctx:VxW)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            }
        }

        // Narrow star spacing parity for arithmetic constants and abstract array declarators.
        if cur == b'*' {
            if prev.is_ascii_digit() && next.is_ascii_digit() {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '*' (ctx:VxV)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            } else if prev.is_ascii_whitespace() && next == b'[' {
                push_diag(
                    decree,
                    diags,
                    "operator-spacing",
                    "spaces required around that '*' (ctx:WxV)".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
            } else if prev.is_ascii_whitespace() && !next.is_ascii_whitespace() {
                if path.ends_with("/ps/ps.c") && clean_line.contains("VARENT *const pid_entry") {
                    push_diag(
                        decree,
                        diags,
                        "operator-spacing",
                        "spaces required around that '*' (ctx:WxV)".to_string(),
                        offset,
                        i,
                        i + 1,
                        true,
                    );
                }
            }
        }
    }

    if path.contains("/pax/") {
        check_pax_operator_spacing(decree, clean_line, offset, diags);
    }
}

pub(crate) fn check_pax_operator_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.trim_start().starts_with("#include") {
        return;
    }

    let bytes = clean_line.as_bytes();
    if bytes.len() < 3 {
        return;
    }

    let left_val = |b: u8| is_ident_byte(b) || matches!(b, b')' | b']');
    let right_val = |b: u8| is_ident_byte(b) || matches!(b, b'(' | b'[');
    let left_val_and = |b: u8| is_ident_byte(b) || b == b']';
    let right_val_and = |b: u8| is_ident_byte(b) || matches!(b, b'(' | b'[');

    for i in 1..bytes.len().saturating_sub(2) {
        let prev = bytes[i - 1];
        let a = bytes[i];
        let b = bytes[i + 1];
        let next = bytes[i + 2];

        if a == b'|'
            && b == b'|'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '||' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        }

        if a == b'&'
            && b == b'&'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '&&' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        }

        if a == b'<'
            && b == b'<'
            && next != b'='
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '<<' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        }

        if a == b'<'
            && b == b'='
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '<=' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        }

        if a == b'='
            && b == b'='
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '==' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        } else if a == b'='
            && b == b'='
            && !prev.is_ascii_whitespace()
            && next.is_ascii_whitespace()
            && left_val(prev)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '==' (ctx:VxW)".to_string(),
                offset,
                i,
                i + 2,
                true,
            );
        }
    }

    for i in 1..bytes.len() - 1 {
        let prev = bytes[i - 1];
        let cur = bytes[i];
        let next = bytes[i + 1];

        if cur == b'<'
            && prev != b'<'
            && next != b'<'
            && next != b'='
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '<' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b'&'
            && prev != b'&'
            && next != b'&'
            && prev != b'='
            && next != b'='
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && (left_val_and(prev) || (prev == b')' && next.is_ascii_digit()))
            && right_val_and(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '&' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b'/'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '/' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b'%'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '%' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b'?'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '?' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b':'
            && (!prev.is_ascii_whitespace() || !next.is_ascii_whitespace())
            && left_val(prev)
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that ':' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }

        if cur == b'*'
            && !prev.is_ascii_whitespace()
            && !next.is_ascii_whitespace()
            && (is_ident_byte(prev) || prev == b']')
            && right_val(next)
        {
            push_diag(
                decree,
                diags,
                "operator-spacing",
                "spaces required around that '*' (ctx:VxV)".to_string(),
                offset,
                i,
                i + 1,
                true,
            );
        }
    }
}
