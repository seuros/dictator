use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_quoted_newline_spacing(
    decree: &FreeBsdDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_block_comment = false;

    while i < bytes.len() {
        if in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                in_block_comment = false;
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
            in_block_comment = true;
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
            if i + 2 < bytes.len()
                && bytes[i] == b' '
                && bytes[i + 1] == b'\\'
                && bytes[i + 2] == b'n'
            {
                push_diag(
                    decree,
                    diags,
                    "quoted-newline-spacing",
                    "unnecessary whitespace before a quoted newline".to_string(),
                    offset,
                    i,
                    i + 1,
                    true,
                );
                return;
            }
            i += 1;
        }
    }
}
