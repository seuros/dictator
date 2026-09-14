use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_pointer_cast_star_spacing(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = clean_line.as_bytes();
    if bytes.len() < 4 {
        return;
    }

    for i in 1..bytes.len() - 1 {
        if bytes[i] != b'*' || bytes[i + 1] != b')' {
            continue;
        }
        if bytes[i - 1].is_ascii_whitespace() {
            continue;
        }

        let mut j = i;
        while j > 0 {
            j -= 1;
            if bytes[j] == b'(' {
                let inner = &clean_line[j + 1..i];
                if !inner.is_empty()
                    && inner
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c.is_ascii_whitespace())
                {
                    push_diag(
                        decree,
                        diags,
                        "pointer-cast-spacing",
                        "\"(foo*)\" should be \"(foo *)\"".to_string(),
                        offset,
                        i,
                        i + 1,
                        true,
                    );
                    return;
                }
                break;
            }
            if !bytes[j].is_ascii_alphanumeric()
                && bytes[j] != b'_'
                && !bytes[j].is_ascii_whitespace()
            {
                break;
            }
        }
    }
}
