use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_signal_handler(
    decree: &FreeBsdDecree,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if clean_line.contains("SIG_IGN") || clean_line.contains("SIG_DFL") {
        return;
    }

    let bytes = clean_line.as_bytes();
    let mut i = 0usize;
    while i + "signal".len() < bytes.len() {
        if !clean_line[i..].starts_with("signal") {
            i += 1;
            continue;
        }
        if i > 0 && is_ident_byte(bytes[i - 1]) {
            i += 1;
            continue;
        }
        let mut j = i + "signal".len();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'(' {
            push_diag(
                decree,
                diags,
                "signal-handler",
                "use sigaction to establish signal handlers; signal is not portable".to_string(),
                offset,
                i,
                i + "signal".len(),
                true,
            );
            return;
        }
        i += 1;
    }
}
