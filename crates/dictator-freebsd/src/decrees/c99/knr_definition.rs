use crate::decrees::support::*;
use crate::{FreeBsdDecree, code_line_mask};
use dictator_decree_abi::Diagnostics;

/// K&R definitions declare parameters between the list and the body. C99
/// removed them.
pub(crate) fn check_c99_knr_definition(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let lines: Vec<&str> = source.lines().collect();
    let code_mask = code_line_mask(source);

    let mut offset = 0usize;
    let mut offsets = Vec::with_capacity(lines.len());
    for line in &lines {
        offsets.push(offset);
        offset += line.len() + 1;
    }

    for i in 0..lines.len() {
        if !code_mask[i] {
            continue;
        }
        let line = lines[i];
        // Definitions start in column 0; indented means a call or a body.
        if line.starts_with([' ', '\t']) || line.starts_with('#') {
            continue;
        }
        let trimmed = line.trim_end();
        if !trimmed.ends_with(')') {
            continue;
        }
        let Some(open) = trimmed.find('(') else {
            continue;
        };
        if find_matching_paren(trimmed, open) != Some(trimmed.len() - 1) {
            continue;
        }
        // Must be a plain name; `DB_SHOW_COMMAND(...)` expands to a definition.
        let name = trimmed[..open].trim_end();
        if name.is_empty() || !name.bytes().all(is_ident_byte) || looks_like_macro_name(name) {
            continue;
        }
        if !is_bare_identifier_list(&trimmed[open + 1..trimmed.len() - 1]) {
            continue;
        }

        // K&R bodies open with the parameter declarations or a lone `{`.
        let mut next = i + 1;
        while next < lines.len() && (!code_mask[next] || lines[next].trim().is_empty()) {
            next += 1;
        }
        if next >= lines.len() {
            continue;
        }
        let follow = lines[next].trim();
        let declares_params = lines[next].starts_with([' ', '\t']) && follow.ends_with(';');
        if !declares_params && follow != "{" {
            continue;
        }

        push_diag(
            decree,
            diags,
            "c99-knr-definition",
            format!(
                "`{name}()` is a K&R definition; C99 wants a prototype with the \
                 parameter types in the parameter list"
            ),
            offsets[i],
            0,
            trimmed.len().max(1),
            true,
        );
    }
}

/// True for `a, b, c`: untyped identifiers. Rejects empty, typed, and
/// macro (`SYSCTL_HANDLER_ARGS`) lists.
fn is_bare_identifier_list(params: &str) -> bool {
    let params = params.trim();
    if params.is_empty() {
        return false;
    }

    params.split(',').all(|param| {
        let param = param.trim();
        !param.is_empty()
            && !param.as_bytes()[0].is_ascii_digit()
            && param.bytes().all(is_ident_byte)
            && param != "void"
            && !is_type_like_word(param)
            && !looks_like_macro_name(param)
    })
}
