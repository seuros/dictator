use crate::FreeBsdDecree;
use dictator_decree_abi::{Decree, Diagnostic, Diagnostics, Span};

// Roughly mirrors checkstyle9.pl sanitization for spacing/keyword checks:
// hide comments and literals but preserve column alignment.
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

pub(crate) fn expanded_len(line: &str) -> usize {
    let mut width = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            width += 8 - (width % 8);
        } else {
            width += 1;
        }
    }
    width
}

pub(crate) fn byte_index_for_column(line: &str, target_col: usize) -> usize {
    let mut width = 0usize;
    for (idx, ch) in line.char_indices() {
        if width >= target_col {
            return idx;
        }
        if ch == '\t' {
            width += 8 - (width % 8);
        } else {
            width += 1;
        }
    }
    line.len()
}

pub(crate) fn is_url_only_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut start = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_alphanumeric() {
            start = idx;
            break;
        }
        if idx + ch.len_utf8() >= trimmed.len() {
            return false;
        }
    }

    let rest = &trimmed[start..];
    (rest.starts_with("http://") || rest.starts_with("https://"))
        && !rest.chars().any(char::is_whitespace)
}

pub(crate) fn is_standalone_string_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') {
        return false;
    }

    let mut escaped = false;
    let mut end = None;

    for (i, ch) in trimmed.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end = Some(i);
            break;
        }
    }

    let Some(end_idx) = end else {
        return false;
    };

    let rest = trimmed[end_idx + 1..].trim();
    if rest.is_empty() || rest == "," {
        return true;
    }

    if let Some(before_semicolon) = rest.strip_suffix(';') {
        let before_semicolon = before_semicolon.trim();
        return before_semicolon.is_empty() || before_semicolon == ")";
    }

    false
}

pub(crate) fn find_keyword_without_space(line: &str, keyword: &str) -> Option<usize> {
    let pattern = format!("{keyword}(");
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

pub(crate) fn find_sizeof_address(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i + 6 <= bytes.len() {
        if &bytes[i..i + 6] == b"sizeof" {
            let mut j = i + 6;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] != b'(' {
                i += 1;
                continue;
            }
            j += 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'&' {
                return Some(i);
            }
        }
        i += 1;
    }

    None
}

pub(crate) fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `word` as a whole identifier: no match inside `pci_register`/`register_t`.
pub(crate) fn find_word(line: &str, word: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut start = 0usize;

    while let Some(pos) = line[start..].find(word) {
        let idx = start + pos;
        let end = idx + word.len();
        let prev_ok = idx == 0 || !is_ident_byte(bytes[idx - 1]);
        let next_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if prev_ok && next_ok {
            return Some(idx);
        }
        start = idx + 1;
    }

    None
}

/// No lowercase — `SYSCTL_HANDLER_ARGS`-style, so a macro, not an identifier.
pub(crate) fn looks_like_macro_name(word: &str) -> bool {
    !word.is_empty() && !word.bytes().any(|b| b.is_ascii_lowercase())
}

/// Member access (`p->register`, `s.auto`) rather than a declaration keyword.
pub(crate) fn is_member_access(line: &str, idx: usize) -> bool {
    let before = line[..idx].trim_end();
    before.ends_with('.') || before.ends_with("->")
}

pub(crate) fn is_define_prefix(ctx_before: &str) -> bool {
    let trimmed = ctx_before.trim_start();
    let Some(rest) = trimmed.strip_prefix('#') else {
        return false;
    };
    let mut it = rest.split_whitespace();
    matches!(it.next(), Some("define")) && it.next().is_none()
}

pub(crate) fn is_elif_prefix_with_name(ctx_before: &str, name: &str) -> bool {
    let combined = format!("{ctx_before}{name}");
    let trimmed = combined.trim_start();
    let Some(rest) = trimmed.strip_prefix('#') else {
        return false;
    };
    let mut it = rest.split_whitespace();
    matches!(it.next(), Some("elif"))
        && matches!(it.next(), Some(n) if n == name)
        && it.next().is_none()
}

pub(crate) fn is_type_like_word(word: &str) -> bool {
    matches!(
        word,
        "void"
            | "char"
            | "short"
            | "int"
            | "long"
            | "float"
            | "double"
            | "bool"
            | "unsigned"
            | "signed"
            | "target_ulong"
            | "target_long"
            | "hwaddr"
    ) || word.ends_with("_t")
        || word.ends_with("_handler")
        || word.ends_with("_handler_fn")
        || {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) if first.is_ascii_uppercase() => {
                    let mut has_lower = false;
                    let mut valid = true;
                    for ch in chars {
                        if !ch.is_ascii_alphanumeric() && ch != '_' {
                            valid = false;
                            break;
                        }
                        if ch.is_ascii_lowercase() {
                            has_lower = true;
                        }
                    }
                    valid && has_lower
                }
                _ => false,
            }
        }
}

pub(crate) fn ctx_ends_with_type(ctx: &str) -> bool {
    let mut toks: Vec<&str> = ctx.split_whitespace().collect();
    while matches!(toks.last(), Some(&"const" | &"volatile")) {
        toks.pop();
    }
    if toks.is_empty() {
        return false;
    }
    if toks.len() >= 2 {
        let n = toks.len();
        if matches!(toks[n - 2], "struct" | "union" | "enum") && is_type_like_word(toks[n - 1]) {
            return true;
        }
    }

    let joined = toks.join(" ");
    if matches!(
        joined.as_str(),
        "unsigned char"
            | "unsigned short"
            | "unsigned int"
            | "unsigned long"
            | "unsigned long int"
            | "unsigned long long"
            | "unsigned long long int"
            | "long int"
            | "long long"
            | "long long int"
    ) {
        return true;
    }

    is_type_like_word(toks[toks.len() - 1])
}

pub(crate) fn is_cast_type(inner: &str) -> bool {
    if inner.is_empty() {
        return false;
    }

    let banned = [
        '{', '}', ';', ',', '+', '-', '/', '%', '&', '|', '!', '<', '>', '=', '?', ':',
    ];
    if inner.chars().any(|c| banned.contains(&c)) {
        return false;
    }

    let mut saw_type_marker = false;
    for tok in inner
        .split(|c: char| c.is_whitespace() || c == '*')
        .filter(|t| !t.is_empty())
    {
        if matches!(
            tok,
            "const"
                | "volatile"
                | "unsigned"
                | "signed"
                | "long"
                | "short"
                | "void"
                | "char"
                | "int"
                | "float"
                | "double"
                | "bool"
                | "struct"
                | "union"
                | "enum"
        ) || tok.ends_with("_t")
        {
            saw_type_marker = true;
            continue;
        }
    }

    saw_type_marker
}

pub(crate) fn is_cast_before_minus(line: &str, minus_idx: usize) -> bool {
    let bytes = line.as_bytes();
    if minus_idx == 0 || bytes[minus_idx - 1] != b')' {
        return false;
    }

    let mut depth = 0i32;
    let mut open = None;
    for p in (0..minus_idx).rev() {
        let b = bytes[p];
        if b == b')' {
            depth += 1;
        } else if b == b'(' {
            depth -= 1;
            if depth == 0 {
                open = Some(p);
                break;
            }
        }
    }

    let Some(open_idx) = open else {
        return false;
    };

    let inner = line[open_idx + 1..minus_idx - 1].trim();
    is_cast_type(inner)
}

pub(crate) fn looks_like_type_context(prefix: &str) -> bool {
    prefix.split_whitespace().any(|tok| {
        matches!(
            tok,
            "const"
                | "volatile"
                | "static"
                | "extern"
                | "typedef"
                | "struct"
                | "union"
                | "enum"
                | "void"
                | "char"
                | "short"
                | "int"
                | "long"
                | "float"
                | "double"
                | "bool"
                | "unsigned"
                | "signed"
        ) || tok.ends_with("_t")
    })
}

pub(crate) fn function_prototype_name(trimmed: &str) -> Option<&str> {
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || !trimmed.ends_with(';')
        || trimmed.contains('{')
        || trimmed.contains('=')
    {
        return None;
    }

    let open = trimmed.find('(')?;
    let close = trimmed.rfind(')')?;
    if close < open {
        return None;
    }

    let before = trimmed[..open].trim_end();
    let mut name_start = before.len();
    let b = before.as_bytes();
    while name_start > 0 && is_ident_byte(b[name_start - 1]) {
        name_start -= 1;
    }
    if name_start == before.len() {
        return None;
    }
    if name_start > 0 {
        let prev = b[name_start - 1];
        if !prev.is_ascii_whitespace() && prev != b'*' {
            return None;
        }
    }
    let name = &before[name_start..];
    let mut prefix = before[..name_start].trim_end();
    while prefix.ends_with('*') {
        prefix = prefix[..prefix.len() - 1].trim_end();
    }
    if prefix.is_empty() {
        return None;
    }

    let prefix = prefix
        .strip_prefix("extern ")
        .or_else(|| prefix.strip_prefix("static "))
        .or_else(|| prefix.strip_prefix("asmlinkage "))
        .unwrap_or(prefix)
        .trim_start();
    if prefix.is_empty() {
        return None;
    }
    if !looks_like_type_context(prefix) {
        return None;
    }

    Some(name)
}

pub(crate) fn find_matching_paren(line: &str, open_idx: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.get(open_idx).copied() != Some(b'(') {
        return None;
    }

    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(open_idx) {
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

pub(crate) fn next_non_ws_from(s: &str, mut idx: usize) -> usize {
    let bytes = s.as_bytes();
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
}

pub(crate) fn find_keyword_condition_end(
    line: &str,
    keyword: &str,
) -> Option<(usize, usize, usize)> {
    let bytes = line.as_bytes();
    let mut start = 0usize;

    while let Some(pos) = line[start..].find(keyword) {
        let kw = start + pos;
        let prev_ok = kw == 0 || !is_ident_byte(bytes[kw - 1]);
        let after_kw = kw + keyword.len();
        let after_ok = after_kw >= bytes.len() || !is_ident_byte(bytes[after_kw]);
        if !prev_ok || !after_ok {
            start = kw + 1;
            continue;
        }

        let open = next_non_ws_from(line, after_kw);
        if open >= bytes.len() || bytes[open] != b'(' {
            start = kw + 1;
            continue;
        }

        if let Some(close) = find_matching_paren(line, open) {
            return Some((kw, open, close));
        }
        return None;
    }

    None
}

pub(crate) fn first_non_ws_col(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

pub(crate) fn statement_line_span(
    clean_lines: &[String],
    start_line: usize,
    start_col: usize,
) -> usize {
    let mut line = start_line;
    let mut col = start_col;
    let mut paren = 0i32;
    let mut bracket = 0i32;

    while line < clean_lines.len() {
        let bytes = clean_lines[line].as_bytes();
        let mut i = col;
        while i < bytes.len() {
            let b = bytes[i];
            match b {
                b'(' => paren += 1,
                b')' => {
                    if paren > 0 {
                        paren -= 1;
                    }
                }
                b'[' => bracket += 1,
                b']' => {
                    if bracket > 0 {
                        bracket -= 1;
                    }
                }
                b';' if paren == 0 && bracket == 0 => {
                    return line - start_line + 1;
                }
                b'{' | b'}' if paren == 0 && bracket == 0 => {
                    return line - start_line + 1;
                }
                _ => {}
            }
            i += 1;
        }
        line += 1;
        col = 0;
    }

    line.saturating_sub(start_line).max(1)
}

pub(crate) fn has_trailing_statement_after_cond(line: &str, close_paren: usize) -> bool {
    if close_paren + 1 >= line.len() {
        return false;
    }

    let mut tail = &line[close_paren + 1..];
    tail = tail.trim_start();
    if tail.is_empty() {
        return false;
    }

    if let Some(rest) = tail.strip_prefix('{') {
        tail = rest.trim_start();
    }
    while let Some(rest) = tail.strip_prefix('\\') {
        tail = rest.trim_start();
    }

    !tail.is_empty()
}

pub(crate) fn is_label_line(trimmed: &str) -> bool {
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    let mut saw_colon = false;
    for ch in chars {
        if ch == ':' {
            saw_colon = true;
            break;
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return false;
        }
    }
    saw_colon
}

pub(crate) fn leading_indent(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    expanded_len(&line[..i])
}

pub(crate) fn macro_body_after_name(trimmed_define: &str) -> Option<&str> {
    let rest = trimmed_define["#define ".len()..].trim_start();
    let bytes = rest.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    if i == 0 {
        return None;
    }

    if i < bytes.len() && bytes[i] == b'(' {
        let mut depth = 0i32;
        let mut j = i;
        while j < bytes.len() {
            if bytes[j] == b'(' {
                depth += 1;
            } else if bytes[j] == b')' {
                depth -= 1;
                if depth == 0 {
                    i = j + 1;
                    break;
                }
            }
            j += 1;
        }
    }
    Some(rest[i..].trim_start())
}

pub(crate) fn macro_has_multiple_statements(body: &str) -> bool {
    if body.contains("while (0)") || body.contains("while(0)") {
        return false;
    }
    let Some(sc) = body.find(';') else {
        return false;
    };
    !body[sc + 1..].trim().is_empty()
}

// One row per Diagnostic field plus the FreeBsdDecree/Diagnostics sinks passed
// through from every call site - splitting into a params struct would just
// move the field list, not shrink it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_diag(
    decree: &FreeBsdDecree,
    diags: &mut Diagnostics,
    rule: &str,
    message: String,
    line_offset: usize,
    start_col: usize,
    end_col: usize,
    _enforced: bool,
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
