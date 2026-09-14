use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_cvs_keywords(
    decree: &FreeBsdDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    for kw in ["$FreeBSD", "$Revision", "$Log", "$Id"] {
        if let Some(col) = line.find(kw) {
            let after = col + kw.len();
            let bytes = line.as_bytes();
            let valid_end = after >= bytes.len()
                || bytes[after] == b'$'
                || !(bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_');
            if valid_end {
                push_diag(
                    decree,
                    diags,
                    "cvs-keywords",
                    "CVS style keyword markers, these will _not_ be updated".to_string(),
                    offset,
                    col,
                    col + kw.len(),
                    true,
                );
            }
        }
    }
}
