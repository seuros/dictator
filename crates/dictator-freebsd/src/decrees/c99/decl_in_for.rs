use crate::decrees::support::*;
use crate::{FreeBsdDecree, code_line_mask};
use dictator_decree_abi::Diagnostics;

/// Integer types worth hoisting into a `for`; a counter is never a pointer or
/// an aggregate.
const COUNTER_TYPES: &[&str] = &[
    "int",
    "unsigned",
    "unsigned int",
    "u_int",
    "long",
    "unsigned long",
    "size_t",
    "ssize_t",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
];

/// A counter declared at the top of the function but live only inside one
/// `for` can be declared in the loop itself, which C99 allows and which keeps
/// the name from outliving its use.  Fires only when the name is unused before
/// the loop and after it, so a counter inspected post-loop (the `for (i = 0;
/// ...) if (...) break;` idiom) keeps its declaration.
pub(crate) fn check_c99_decl_in_for(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let lines: Vec<&str> = source.lines().collect();
    let code_mask = code_line_mask(source);

    let offsets = line_offsets(&lines);

    let clean = sanitized_lines(&lines);

    for (body_start, body_end) in function_bodies(&clean, &code_mask) {
        for decl in body_start..body_end {
            if !code_mask[decl] {
                continue;
            }
            let Some(name) = counter_declaration(&clean[decl]) else {
                continue;
            };

            let uses: Vec<usize> = (body_start..body_end)
                .filter(|&i| i != decl && code_mask[i] && find_word(&clean[i], name).is_some())
                .collect();
            if uses.is_empty() {
                continue;
            }

            let loops: Vec<usize> = uses
                .iter()
                .copied()
                .filter(|&i| for_init_binds(&clean[i], name))
                .collect();
            let [for_line] = loops[..] else {
                continue;
            };

            let Some(for_end) = loop_end(&clean, for_line, body_end) else {
                continue;
            };
            if uses.iter().any(|&i| i < for_line || i > for_end) {
                continue;
            }

            let indent = first_non_ws_col(&clean[decl]);
            push_diag(
                decree,
                diags,
                "c99-decl-in-for",
                format!(
                    "`{name}` is live only in the loop below; C99 allows \
                     declaring it in the `for` statement"
                ),
                offsets[decl],
                indent,
                lines[decl].len().max(indent + 1),
                true,
            );
        }
    }
}

/// Half-open line ranges of top-level function bodies: a lone `{` in column 0
/// through its matching `}`.
fn function_bodies(clean: &[String], code_mask: &[bool]) -> Vec<(usize, usize)> {
    let mut bodies = Vec::new();
    let mut open = None;
    let mut depth = 0i32;

    for (i, line) in clean.iter().enumerate() {
        if !code_mask[i] {
            continue;
        }
        if open.is_none() {
            if line.trim_end() == "{" {
                open = Some(i + 1);
                depth = 1;
            }
            continue;
        }

        depth += brace_delta(line);
        if depth == 0
            && let Some(start) = open.take()
        {
            bodies.push((start, i));
        }
    }

    bodies
}

fn brace_delta(line: &str) -> i32 {
    line.bytes().fold(0i32, |acc, b| match b {
        b'{' => acc + 1,
        b'}' => acc - 1,
        _ => acc,
    })
}

/// The single name declared by an uninitialized counter declaration, e.g.
/// `\tint i;`.  Lists (`int i, j;`) and initializers are left alone.
fn counter_declaration(clean_line: &str) -> Option<&str> {
    let trimmed = clean_line.trim();
    let body = trimmed.strip_suffix(';')?;
    if body.contains([',', '=', '*', '(', '[']) {
        return None;
    }

    let mut toks: Vec<&str> = body.split_whitespace().collect();
    let name = toks.pop()?;
    if toks.is_empty() || !name.bytes().all(is_ident_byte) || looks_like_macro_name(name) {
        return None;
    }
    if !COUNTER_TYPES.contains(&toks.join(" ").as_str()) {
        return None;
    }

    let start = body.len() - name.len();
    Some(&body[start..])
}

/// True when the line is a `for` whose init assigns to `name`.
fn for_init_binds(clean_line: &str, name: &str) -> bool {
    let Some((_, open, _)) = find_keyword_condition_end(clean_line, "for") else {
        return false;
    };

    let init = &clean_line[open + 1..];
    let init = init.split(';').next().unwrap_or(init).trim();
    let Some(rest) = init.strip_prefix(name) else {
        return false;
    };
    let rest = rest.trim_start();
    rest.starts_with('=') && !rest.starts_with("==")
}

/// Last line of the loop that starts on `for_line`.
fn loop_end(clean: &[String], for_line: usize, body_end: usize) -> Option<usize> {
    let line = &clean[for_line];
    let (_, open, close) = find_keyword_condition_end(line, "for")?;
    let _ = open;

    if line[close + 1..].trim_start().starts_with('{') {
        let mut depth = 0i32;
        for (i, line) in clean.iter().enumerate().take(body_end).skip(for_line) {
            depth += brace_delta(line);
            if depth == 0 {
                return Some(i);
            }
        }
        return None;
    }

    let span = statement_line_span(clean, for_line, close + 1);
    Some((for_line + span - 1).min(body_end - 1))
}
