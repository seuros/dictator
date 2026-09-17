#![allow(unexpected_cfgs)] // state_machine! macro uses cfg(inspect) internally

use dictator_decree_abi::{BoxDecree, Capability, Decree, DecreeMetadata, Diagnostics};
use state_machines::state_machine;

mod decrees;
use decrees::*;

// Tracks whether we are in normal code or inside a `/* */` block comment.
// Used by multi-line checks that must skip commented-out code.
state_machine! {
    name: CParser,
    initial: Code,
    dynamic: true,
    states: [Code, BlockComment],
    events {
        open_block_comment {
            transition: { from: Code, to: BlockComment }
        }
        close_block_comment {
            transition: { from: BlockComment, to: Code }
        }
    }
}

/// Returns one `bool` per source line: `true` = normal code, `false` = inside a block comment.
/// Single-line `/* ... */` comments keep the line marked as code.
fn code_line_mask(source: &str) -> Vec<bool> {
    let mut parser = DynamicCParser::new(());
    source
        .lines()
        .map(|line| {
            let is_code = parser.current_state() == CParserState::Code;
            if is_code {
                if let Some(pos) = line.find("/*")
                    && !line[pos + 2..].contains("*/")
                {
                    let _ = parser.handle(CParserEvent::OpenBlockComment);
                }
            } else if line.contains("*/") {
                let _ = parser.handle(CParserEvent::CloseBlockComment);
            }
            is_code
        })
        .collect()
}

#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    FreeBsdDecree.lint("test.c", source)
}

/// mdoc(7) section names in their canonical man(7) order.
const MAN_CANONICAL_SECTIONS: &[&str] = &[
    "NAME",
    "LIBRARY",
    "SYNOPSIS",
    "DESCRIPTION",
    "CONTEXT",
    "HARDWARE",
    "IMPLEMENTATION NOTES",
    "RETURN VALUES",
    "ENVIRONMENT",
    "FILES",
    "EXIT STATUS",
    "EXAMPLES",
    "DIAGNOSTICS",
    "ERRORS",
    "SEE ALSO",
    "STANDARDS",
    "HISTORY",
    "AUTHORS",
    "CAVEATS",
    "BUGS",
    "SECURITY CONSIDERATIONS",
];

fn is_man_page(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| ext.chars().next())
        .is_some_and(|c| c.is_ascii_digit() && c != '0')
}

fn man_stem(path: &str) -> Option<&str> {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
}

fn lint_man_page(decree: &FreeBsdDecree, path: &str, source: &str, diags: &mut Diagnostics) {
    let mut offset = 0usize;
    for raw in source.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        check_line_length(decree, line, false, offset, diags);
        check_man_trailing_whitespace(decree, line, offset, diags);
        offset += raw.len();
    }

    check_man_required_macros(decree, path, source, diags);
    check_man_section_order(decree, source, diags);
    check_man_name_section_format(decree, source, diags);
    check_man_nd_quoted(decree, source, diags);
    check_man_cd_quoted(decree, source, diags);
    check_man_legacy_divider(decree, source, diags);
    check_man_verbose_license(decree, source, diags);
    check_man_dl_quoted(decree, source, diags);
    check_man_apostrophe_literal(decree, source, diags);
    check_man_va_numeric_placeholder(decree, source, diags);
    check_man_module_load_boilerplate(decree, source, diags);
    check_man_date_format(decree, source, diags);
    check_banned_gpl_license(decree, source, diags);
    check_man_spdx_order(decree, source, diags);
}

pub struct FreeBsdDecree;

impl Decree for FreeBsdDecree {
    fn name(&self) -> &str {
        "freebsd"
    }

    fn metadata(&self) -> DecreeMetadata {
        DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "FreeBSD style(9) enforcement for kernel C code".to_string(),
            persona: "Beastie".to_string(),
            dectauthors: Some("Abdelkader Boudih <terminale@gmail.com>".to_string()),
            supported_extensions: vec![
                "c".to_string(),
                "h".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
            ],
            supported_filenames: vec![],
            skip_filenames: vec![],
            file_scope_rules: vec![
                "man-missing-macro".to_string(),
                "man-missing-section".to_string(),
                "man-name-mismatch".to_string(),
            ],
            capabilities: vec![Capability::Lint],
        }
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();

        if is_man_page(path) {
            lint_man_page(self, path, source, &mut diags);
            return diags;
        }

        check_banned_gpl_license(self, source, &mut diags);

        let mut offset = 0usize;
        let mut in_block_comment = false;
        let mut in_block_comment_printf = false;
        let in_tests_tree = path.contains("/tests/");

        for raw in source.split_inclusive('\n') {
            let line = raw.strip_suffix('\n').unwrap_or(raw);
            let clean = sanitize_code_line(line, &mut in_block_comment);

            check_return_parens(self, &clean, line, offset, &mut diags);
            check_keyword_space_before_paren(self, &clean, line, offset, &mut diags);
            check_function_name_space(self, &clean, line, offset, &mut diags);
            check_line_length(self, line, in_tests_tree, offset, &mut diags);
            check_cvs_keywords(self, line, offset, &mut diags);
            check_comma_spacing(self, &clean, line, offset, &mut diags);
            check_close_brace_spacing_edge(self, &clean, offset, &mut diags);
            check_open_paren_spacing(self, &clean, line, offset, &mut diags);
            check_parenthesis_spacing(self, &clean, line, offset, &mut diags);
            check_or_assign_spacing(self, &clean, offset, &mut diags);
            check_equal_spacing_narrow(self, &clean, offset, &mut diags);
            check_and_spacing_edge(self, &clean, offset, &mut diags);
            check_basic_operator_spacing(self, path, &clean, line, offset, &mut diags);
            check_pointer_cast_star_spacing(self, &clean, offset, &mut diags);
            check_pointer_binding_spacing(self, &clean, offset, &mut diags);
            check_double_semicolon(self, &clean, line, offset, &mut diags);
            check_malformed_include(self, line, offset, &mut diags);
            check_dead_code(self, &clean, offset, &mut diags);
            check_sizeof_address(self, &clean, offset, &mut diags);
            check_function_name(self, &clean, offset, &mut diags);
            check_signal_handler(self, &clean, offset, &mut diags);
            check_extern_in_c(self, path, &clean, offset, &mut diags);
            check_printf_format(self, line, &mut in_block_comment_printf, offset, &mut diags);
            check_quoted_newline_spacing(self, line, offset, &mut diags);

            // C99 spellings over their C89/GNU predecessors.
            check_c99_obsolete_storage_class(self, &clean, offset, &mut diags);
            check_c99_empty_param_list(self, &clean, offset, &mut diags);
            check_c99_legacy_int_types(self, &clean, offset, &mut diags);
            check_c99_bool_macros(self, &clean, offset, &mut diags);
            check_c99_gnu_inline(self, &clean, offset, &mut diags);
            check_c99_named_variadic_macro(self, &clean, offset, &mut diags);

            offset += raw.len();
        }

        // Multi-line checks that need full-source context.
        check_block_comment_style(self, source, &mut diags);
        check_trailing_statements(self, source, &mut diags);
        check_switch_case_indent(self, source, &mut diags);
        check_braces_encouraged(self, source, &mut diags);
        check_open_brace_placement(self, source, &mut diags);
        check_initializer_open_brace(self, source, &mut diags);
        check_conditional_indent(self, source, &mut diags);
        check_conditional_indent_edges(self, source, &mut diags);
        check_trailing_statements_preproc_split(self, source, &mut diags);
        check_macro_complex_values(self, source, &mut diags);
        check_else_follow_close_brace(self, path, source, &mut diags);
        check_macro_statement_wrapping(self, path, source, &mut diags);
        check_c99_knr_definition(self, source, &mut diags);

        diags
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(FreeBsdDecree)
}

#[unsafe(no_mangle)]
pub fn dictator_create_decree() -> BoxDecree {
    init_decree()
}
