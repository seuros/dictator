use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// `#define TRUE`/`FALSE`/`BOOL` — C99 has <stdbool.h>.
pub(crate) fn check_c99_bool_macros(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = clean_line.trim_start();
    let Some(rest) = trimmed.strip_prefix('#') else {
        return;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix("define") else {
        return;
    };

    let name = rest
        .trim_start()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next();
    let Some(name) = name.filter(|n| matches!(*n, "TRUE" | "FALSE" | "BOOL" | "BOOLEAN")) else {
        return;
    };

    let col = find_word(clean_line, name).unwrap_or(0);
    push_diag(
        decree,
        diags,
        "c99-bool-macros",
        format!(
            "`{name}` is a hand-rolled truth value; C99 has <stdbool.h> with \
             `bool`, `true`, and `false`"
        ),
        offset,
        col,
        col + name.len(),
        true,
    );
}
