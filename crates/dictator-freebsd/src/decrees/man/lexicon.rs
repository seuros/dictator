use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// Phrases the manual has a shorter idiom for.  Matching is per line, so a
// phrase broken across a line boundary is missed rather than guessed at.
const LEXICON: &[(&str, &str)] = &[
    ("Mac computers", "Macs"),
    ("Macintosh computers", "Macs"),
    ("Apple computers", "Macs"),
];

pub(crate) fn check_man_lexicon(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);

        if !line.trim_start().starts_with(".\\\"") {
            let lowered = line.to_ascii_lowercase();

            for (phrase, preferred) in LEXICON {
                let needle = phrase.to_ascii_lowercase();
                if let Some(col) = lowered.find(&needle) {
                    push_diag(
                        decree,
                        diags,
                        "man-lexicon",
                        format!("say \"{preferred}\" rather than \"{phrase}\""),
                        offset,
                        col,
                        col + phrase.len(),
                        true,
                    );
                }
            }
        }

        offset += raw.len();
    }
}
