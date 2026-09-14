use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_extern_in_c(
    decree: &FreeBsdDecree,
    path: &str,
    clean_line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    if !path.ends_with(".c") {
        return;
    }
    let trimmed = clean_line.trim_start();
    let first_kw = trimmed.split_whitespace().next().unwrap_or("");
    let is_non_static_proto = first_kw != "static"
        && first_kw != "asmlinkage"
        && first_kw != "typedef"
        && matches!(
            function_prototype_name(trimmed),
            Some(name) if name != "uninitialized_var"
        );
    if first_kw == "extern" || is_non_static_proto {
        let col = clean_line.find("extern").unwrap_or(0);
        push_diag(
            decree,
            diags,
            "extern-in-c",
            "externs should be avoided in .c files".to_string(),
            offset,
            col,
            col + "extern".len(),
            true,
        );
    }
}
