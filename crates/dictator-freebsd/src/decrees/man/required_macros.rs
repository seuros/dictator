use crate::decrees::support::*;
use crate::{FreeBsdDecree, man_stem};
use dictator_decree_abi::Diagnostics;

pub(crate) fn check_man_required_macros(
    decree: &FreeBsdDecree,
    path: &str,
    source: &str,
    diags: &mut Diagnostics,
) {
    let mut found_dd = false;
    let mut found_dt = false;
    let mut found_os = false;
    let mut found_sh_name = false;
    let mut found_sh_synopsis = false;
    let mut found_sh_description = false;
    let mut first_nm_arg: Option<&str> = None;

    let mut prologue: Vec<&str> = Vec::new();
    let mut offset = 0usize;
    let mut header_order_offset = 0usize;
    let mut header_order_len = 0usize;

    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let trimmed = line.trim();

        if let Some(macro_name) = trimmed.split_whitespace().next() {
            match macro_name {
                ".Dd" => found_dd = true,
                ".Dt" => found_dt = true,
                ".Os" => found_os = true,
                ".Nm" if first_nm_arg.is_none() => {
                    first_nm_arg = trimmed.split_whitespace().nth(1);
                }
                _ => {}
            }
        }

        if trimmed == ".Sh NAME" {
            found_sh_name = true;
        } else if trimmed == ".Sh SYNOPSIS" {
            found_sh_synopsis = true;
        } else if trimmed == ".Sh DESCRIPTION" {
            found_sh_description = true;
        }

        if prologue.len() < 3 && !trimmed.is_empty() && !trimmed.starts_with(".\\\"") {
            if prologue.is_empty() {
                header_order_offset = offset;
            }
            prologue.push(trimmed.split_whitespace().next().unwrap_or(""));
            header_order_len = offset + line.len() - header_order_offset;
        }

        offset += raw.len();
    }

    for (name, present) in [(".Dd", found_dd), (".Dt", found_dt), (".Os", found_os)] {
        if !present {
            push_diag(
                decree,
                diags,
                "man-missing-macro",
                format!("man page is missing the {name} macro"),
                0,
                0,
                1,
                true,
            );
        }
    }

    if found_dd && found_dt && found_os && prologue.as_slice() != [".Dd", ".Dt", ".Os"] {
        push_diag(
            decree,
            diags,
            "man-header-order",
            "the .Dd, .Dt, .Os prologue macros must appear first, in that order".to_string(),
            header_order_offset,
            0,
            header_order_len.max(1),
            true,
        );
    }

    for (name, present) in [
        ("NAME", found_sh_name),
        ("SYNOPSIS", found_sh_synopsis),
        ("DESCRIPTION", found_sh_description),
    ] {
        if !present {
            push_diag(
                decree,
                diags,
                "man-missing-section",
                format!("man page is missing the .Sh {name} section"),
                0,
                0,
                1,
                true,
            );
        }
    }

    if let (Some(stem), Some(nm_arg)) = (man_stem(path), first_nm_arg)
        && stem != nm_arg
    {
        push_diag(
            decree,
            diags,
            "man-name-mismatch",
            format!(".Nm argument '{nm_arg}' does not match the filename '{stem}'"),
            0,
            0,
            1,
            true,
        );
    }
}
