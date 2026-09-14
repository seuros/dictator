use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// mdoc(7) specifies `.Dd` as either the OpenBSD `$Mdocdate$` cvs-expansion
// keyword or a literal `month day, year` date (full English month name,
// integer day, 4-digit year), e.g. `.Dd January 1, 2024`. Parse the latter
// with jiff to reject malformed or calendar-invalid dates (e.g. `Jan 1 2024`,
// `February 31, 2024`) that string-shape checks alone would miss.
pub(crate) fn check_man_date_format(decree: &FreeBsdDecree, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix(".Dd") {
            let arg = rest.trim();

            if !arg.is_empty()
                && !arg.starts_with("$Mdocdate")
                && let Err(e) = jiff::civil::Date::strptime("%B %d, %Y", arg)
            {
                push_diag(
                    decree,
                    diags,
                    "man-date-format",
                    format!("malformed .Dd date; expected \"Month D, YYYY\": {e}"),
                    offset,
                    0,
                    line.len().max(1),
                    true,
                );
            }
        }

        offset += raw.len();
    }
}
