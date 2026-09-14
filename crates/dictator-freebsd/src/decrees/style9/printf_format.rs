use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_printf_format(
    decree: &FreeBsdDecree,
    line: &str,
    in_block_comment: &mut bool,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        if *in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                *in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            break;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            *in_block_comment = true;
            i += 2;
            continue;
        }

        if bytes[i] != b'"' {
            i += 1;
            continue;
        }

        i += 1;
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'\\' {
                i += 2;
                continue;
            }

            if bytes[i] == b'"' {
                i += 1;
                break;
            }

            if bytes[i] == b'%' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
                    i += 2;
                    continue;
                }

                if i + 2 < bytes.len()
                    && bytes[i + 1] == b'L'
                    && matches!(bytes[i + 2], b'u' | b'd' | b'i')
                {
                    push_diag(
                        decree,
                        diags,
                        "printf-format",
                        "%Ld/%Lu are not-standard C, use %lld/%llu".to_string(),
                        offset,
                        i,
                        i + 3,
                        true,
                    );
                    return;
                }
            }

            i += 1;
        }
    }
}
