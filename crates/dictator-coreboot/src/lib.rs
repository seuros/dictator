use dictator_decree_abi::{BoxDecree, Capability, Decree, DecreeMetadata, Diagnostics};

mod decrees;
use decrees::*;

pub struct CorebootDecree;

/// Lint a source string as `test.c`. Convenience entry point for tests.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    CorebootDecree.lint("test.c", source)
}

impl Decree for CorebootDecree {
    fn name(&self) -> &str {
        "coreboot"
    }

    fn metadata(&self) -> DecreeMetadata {
        DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "coreboot coding style enforcement for firmware C code".to_string(),
            dectauthors: Some("Abdelkader Boudih <terminale@gmail.com>".to_string()),
            supported_extensions: vec!["c".to_string(), "h".to_string()],
            supported_filenames: vec![],
            skip_filenames: vec![],
            capabilities: vec![Capability::Lint],
        }
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();
        let mut offset = 0usize;
        let mut in_block_comment = false;

        for raw in source.split_inclusive('\n') {
            let line = raw.strip_suffix('\n').unwrap_or(raw);
            let clean = sanitize_code_line(line, &mut in_block_comment);

            check_non_ascii(self, line, offset, &mut diags);
            check_no_printf(self, &clean, offset, &mut diags);
            check_printk_log_level(self, &clean, offset, &mut diags);
            check_config_macro(self, line, offset, &mut diags);
            check_no_auto_includes(self, line, offset, &mut diags);
            check_function_name(self, &clean, offset, &mut diags);
            check_free_is_noop(self, &clean, offset, &mut diags);
            check_die_usage(self, &clean, offset, &mut diags);
            check_style_labels(self, &clean, line, offset, &mut diags);
            check_coreboot_lowercase(self, line, offset, &mut diags);

            offset += raw.len();
        }

        diags
    }
}

#[unsafe(no_mangle)]
pub fn dictator_create_decree() -> BoxDecree {
    Box::new(CorebootDecree)
}
