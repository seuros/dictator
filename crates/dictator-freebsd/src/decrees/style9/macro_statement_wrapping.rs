use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_macro_statement_wrapping(
    decree: &FreeBsdDecree,
    path: &str,
    source: &str,
    diags: &mut Diagnostics,
) {
    if !path.contains("/pax/") && !path.ends_with("/ps/keyword.c") {
        return;
    }

    let mut offset = 0usize;
    let mut in_define_cont = false;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim_start();

        if trimmed.starts_with("#define ") {
            if let Some(body) = macro_body_after_name(trimmed)
                && macro_has_multiple_statements(body)
            {
                let col = line.find("#define").unwrap_or(0);
                push_diag(
                    decree,
                    diags,
                    "macro-do-while",
                    "Macros with multiple statements should be enclosed in a do - while loop"
                        .to_string(),
                    offset,
                    col,
                    col + "#define".len(),
                    true,
                );
            }
            in_define_cont = line.trim_end().ends_with('\\');
            offset += raw.len();
            continue;
        }

        if in_define_cont && trimmed.contains("} while (0);") {
            let col = line.find("while").unwrap_or(0);
            push_diag(
                decree,
                diags,
                "macro-while-zero-semicolon",
                "suspicious ; after while (0)".to_string(),
                offset,
                col,
                col + "while".len(),
                true,
            );
        }

        in_define_cont = in_define_cont && line.trim_end().ends_with('\\');
        offset += raw.len();
    }
}
