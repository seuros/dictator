use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// Closing punctuation on a macro line has to be its own argument, otherwise
// it is typeset as part of the preceding word and inherits that word's font:
// `.Pq 2018 and later.` sets the period inside the parentheses.  The tree is
// near-unanimous (3257 separated against 37 attached).
const ABBREVIATIONS: &[&str] = &[
    "e.g", "i.e", "etc", "vs", "cf", "al", "Inc", "Ltd", "Co", "Jr", "Sr", "Dr", "St", "approx",
];

pub(crate) fn check_man_macro_attached_period(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim_end();

        if let Some(col) = attached_period_col(trimmed) {
            push_diag(
                decree,
                diags,
                "man-macro-attached-period",
                "separate trailing punctuation from the preceding word with a space".to_string(),
                offset,
                col,
                trimmed.len().max(col + 1),
                true,
            );
        }

        offset += raw.len();
    }
}

// Byte column of the offending period, or None when the line is not a macro
// line, carries no trailing period, or already separates it.
fn attached_period_col(line: &str) -> Option<usize> {
    let rest = line.strip_prefix('.')?;
    let mut chars = rest.chars();
    if !chars.next()?.is_ascii_uppercase() || !chars.next()?.is_ascii_lowercase() {
        return None;
    }

    let last = line.split_whitespace().next_back()?;
    let stem = last.strip_suffix('.')?;
    if stem.is_empty() || ABBREVIATIONS.contains(&stem) {
        return None;
    }

    let tail = stem.chars().next_back()?;
    if !tail.is_ascii_alphanumeric() && tail != ')' {
        return None;
    }

    Some(line.len() - 1)
}
