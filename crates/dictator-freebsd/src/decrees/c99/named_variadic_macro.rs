use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// `#define f(fmt, args...)` — a GNU extension predating `__VA_ARGS__`.
pub(crate) fn check_c99_named_variadic_macro(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = clean_line.trim_start();
    if !trimmed
        .strip_prefix('#')
        .is_some_and(|rest| rest.trim_start().starts_with("define"))
    {
        return;
    }

    let Some(open) = clean_line.find('(') else {
        return;
    };
    let Some(close) = find_matching_paren(clean_line, open) else {
        return;
    };

    let params = &clean_line[open + 1..close];
    let Some(dots) = params.find("...") else {
        return;
    };
    // `__VA_ARGS__` style is `(fmt, ...)`: only a separator before the dots.
    if !params[..dots]
        .chars()
        .next_back()
        .is_some_and(|c| is_ident_byte(c as u8) && c.is_ascii())
    {
        return;
    }

    let col = open + 1 + dots;
    push_diag(
        decree,
        diags,
        "c99-named-variadic-macro",
        "named variadic macro parameters are a GNU extension; use `...` with \
         `__VA_ARGS__`"
            .to_string(),
        offset,
        col,
        col + 3,
        true,
    );
}
