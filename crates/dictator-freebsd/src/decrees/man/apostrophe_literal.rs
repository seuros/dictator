use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// mdoc(7) provides `.Ap` specifically to insert a possessive/plural
// apostrophe without surrounding whitespace after a macro like `.Xr`; a
// literal `'s` following the inline `Ns` call is the pre-.Ap workaround, e.g.
// `.Xr ichwd 4 Ns 's` should be `.Xr ichwd 4 Ns Ap s`. `Ns` here is a
// macro-as-argument token (no leading dot), not a line-leading request.
pub(crate) fn check_man_apostrophe_literal(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let flagged = tokens
            .windows(2)
            .any(|w| w[0] == "Ns" && w[1].starts_with('\''));

        if flagged {
            push_diag(
                decree,
                diags,
                "man-apostrophe-literal",
                "use Ap instead of a literal apostrophe after Ns".to_string(),
                offset,
                0,
                line.len().max(1),
                true,
            );
        }

        offset += raw.len();
    }
}
