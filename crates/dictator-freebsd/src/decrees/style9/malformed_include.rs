use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_malformed_include(
    decree: &FreeBsdDecree,
    line: &str,
    offset: usize,
    diags: &mut Diagnostics,
) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("#") || !trimmed.contains("include") {
        return;
    }

    let include_pos = trimmed.find("include").unwrap_or(0);
    let after_include = trimmed[include_pos + "include".len()..].trim_start();
    if (after_include.starts_with('<') || after_include.starts_with('"'))
        && after_include.contains("//")
    {
        let col = line.find("#").unwrap_or(0);
        push_diag(
            decree,
            diags,
            "malformed-include",
            "malformed #include filename".to_string(),
            offset,
            col,
            col + 8,
            true,
        );
    }
}
