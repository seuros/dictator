use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

/// Proper nouns the manual and comments should capitalize.  Comment prose
/// only, so an identifier or a macro keeps whatever case the vendor chose.
/// Add an entry only once the tree has been checked for a competing spelling
/// — a lexicon that argues with existing code is noise.
const PROPER_NOUNS: &[(&str, &str)] = &[("retina", "Retina")];

pub(crate) fn check_lexicon(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    for (offset, text) in comment_regions(source) {
        for (lower, proper) in PROPER_NOUNS {
            let mut start = 0usize;
            while let Some(pos) = find_word(&text[start..], lower) {
                let idx = start + pos;
                push_diag(
                    decree,
                    diags,
                    "lexicon",
                    format!("`{proper}` is a proper noun; capitalize it"),
                    offset + idx,
                    0,
                    lower.len(),
                    true,
                );
                start = idx + lower.len();
            }
        }
    }
}

/// Comment text with the absolute byte offset of each region, so spans point
/// at the prose rather than at the line.
fn comment_regions(source: &str) -> Vec<(usize, &str)> {
    let bytes = source.as_bytes();
    let mut regions = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i + 2;
            let mut end = start;
            while end + 1 < bytes.len() && !(bytes[end] == b'*' && bytes[end + 1] == b'/') {
                end += 1;
            }
            let end = end.min(bytes.len());
            regions.push((start, &source[start..end]));
            i = end + 2;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            let start = i + 2;
            let end = source[start..]
                .find('\n')
                .map_or(bytes.len(), |nl| start + nl);
            regions.push((start, &source[start..end]));
            i = end;
            continue;
        }

        i += 1;
    }

    regions
}
