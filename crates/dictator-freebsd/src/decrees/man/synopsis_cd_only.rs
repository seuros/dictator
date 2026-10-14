use crate::FreeBsdDecree;
use crate::decrees::support::*;
use dictator_decree_abi::Diagnostics;

// A device driver SYNOPSIS is a bare list of `.Cd` lines; the "To compile
// this driver into the kernel, place the following lines in your kernel
// configuration file" prose and its surrounding `.Bd`/`.Ed` display are
// noise the reader already understands from the section.
//
// Gated on a SYNOPSIS that has `.Cd` and none of `.Fn`/`.Ft`/`.In`, so
// kernel-interface pages (netgraph nodes, bus APIs) that document both a
// config line and a function prototype are left alone.
pub(crate) fn check_man_synopsis_cd_only(
    decree: &FreeBsdDecree,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut offset = 0usize;
    let mut in_synopsis = false;
    let mut body: Vec<(usize, &str)> = Vec::new();
    let mut has_cd = false;
    let mut has_interface = false;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if trimmed.starts_with(".Sh ") {
            if in_synopsis {
                break;
            }
            in_synopsis = trimmed == ".Sh SYNOPSIS";
            offset += raw.len();
            continue;
        }

        if in_synopsis {
            if trimmed.starts_with(".Cd") {
                has_cd = true;
            } else if trimmed.starts_with(".Fn")
                || trimmed.starts_with(".Ft")
                || trimmed.starts_with(".In")
            {
                has_interface = true;
            } else if !trimmed.is_empty() && !trimmed.starts_with(".\\\"") {
                body.push((offset, line));
            }
        }

        offset += raw.len();
    }

    if !has_cd || has_interface {
        return;
    }

    for (line_offset, line) in body {
        push_diag(
            decree,
            diags,
            "man-synopsis-cd-only",
            "a driver SYNOPSIS holds only `.Cd` lines; drop this".to_string(),
            line_offset,
            0,
            line.len().max(1),
            true,
        );
    }
}
