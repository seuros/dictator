#![warn(rust_2024_compatibility, clippy::all)]

//! decree.python - Python structural rules (PEP 8 compliant).

mod file_length;
mod imports;
mod indentation;

use dictator_decree_abi::{BoxDecree, Decree, Diagnostics};
use dictator_supreme::SupremeConfig;

pub use imports::{ImportType, classify_module, is_python_stdlib};

/// Configuration for python decree
#[derive(Debug, Clone)]
pub struct PythonConfig {
    pub max_lines: usize,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            max_lines: file_length::DEFAULT_MAX_LINES,
        }
    }
}

#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_config(source, &PythonConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &PythonConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    file_length::check_file_line_count(source, config.max_lines, &mut diags);
    imports::check_import_ordering(source, &mut diags);
    indentation::check_indentation_consistency(source, &mut diags);

    diags
}

#[derive(Default)]
pub struct Python {
    config: PythonConfig,
    supreme: SupremeConfig,
}

impl Python {
    #[must_use]
    pub const fn new(config: PythonConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        let mut diags = dictator_supreme::lint_source_with_owner(source, &self.supreme, "python");
        diags.extend(lint_source_with_config(source, &self.config));
        diags
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Python structural rules".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["py".to_string()],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(Python::default())
}

/// Create decree with custom config
#[must_use]
pub fn init_decree_with_config(config: PythonConfig) -> BoxDecree {
    Box::new(Python::new(config, SupremeConfig::default()))
}

/// Create decree with custom config + supreme config (merged from decree.supreme + decree.python)
#[must_use]
pub fn init_decree_with_configs(config: PythonConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(Python::new(config, supreme))
}

/// Convert `DecreeSettings` to `PythonConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> PythonConfig {
    PythonConfig {
        max_lines: settings.max_lines.unwrap_or(file_length::DEFAULT_MAX_LINES),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_file_too_long() {
        use std::fmt::Write;
        let mut src = String::new();
        for i in 0..400 {
            let _ = writeln!(src, "x = {i}");
        }
        let diags = lint_source(&src);
        assert!(
            diags.iter().any(|d| d.rule == "python/file-too-long"),
            "Should detect file with >380 code lines"
        );
    }

    #[test]
    fn ignores_comments_in_line_count() {
        use std::fmt::Write;
        let mut src = String::new();
        for i in 0..380 {
            let _ = writeln!(src, "x = {i}");
        }
        for i in 0..60 {
            let _ = writeln!(src, "# Comment {i}");
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "python/file-too-long"),
            "Should not count comment-only lines"
        );
    }

    #[test]
    fn ignores_blank_lines_in_count() {
        use std::fmt::Write;
        let mut src = String::new();
        for i in 0..380 {
            let _ = writeln!(src, "x = {i}");
        }
        for _ in 0..60 {
            src.push('\n');
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "python/file-too-long"),
            "Should not count blank lines"
        );
    }

    #[test]
    fn detects_wrong_import_order_stdlib_after_third_party() {
        let src = r"
import requests
import os
import sys
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "python/import-order"),
            "Should detect stdlib import after third-party import"
        );
    }

    #[test]
    fn detects_wrong_import_order_local_before_third_party() {
        let src = r"
from . import config
import requests
import os
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "python/import-order"),
            "Should detect wrong import order"
        );
    }

    #[test]
    fn accepts_correct_import_order() {
        let src = r"
import os
import sys
import json
from typing import Dict, List
import requests
import django
from . import config
from .utils import helper
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "python/import-order"),
            "Should accept correct import order"
        );
    }

    #[test]
    fn detects_mixed_tabs_and_spaces() {
        let src = "def test():\n\tx = 1\n  y = 2\n";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "python/mixed-indentation"),
            "Should detect mixed tabs and spaces"
        );
    }

    #[test]
    fn detects_inconsistent_indentation_depth() {
        let src = r"
def test():
  if True:
     x = 1
  y = 2
";
        let diags = lint_source(src);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "python/inconsistent-indentation"),
            "Should detect inconsistent indentation depth (3 spaces instead of 2 or 4)"
        );
    }

    #[test]
    fn accepts_consistent_indentation() {
        let src = r"
def test():
    if True:
        x = 1
        y = 2
    z = 3
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "python/mixed-indentation"
                || d.rule == "python/inconsistent-indentation"),
            "Should accept consistent indentation"
        );
    }

    #[test]
    fn handles_empty_file() {
        let src = "";
        let diags = lint_source(src);
        assert!(diags.is_empty(), "Empty file should have no violations");
    }

    #[test]
    fn handles_file_with_only_comments() {
        let src = "# Comment 1\n# Comment 2\n# Comment 3\n";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "python/file-too-long"),
            "File with only comments should not trigger line count"
        );
    }

    #[test]
    fn detects_stdlib_correctly() {
        assert!(is_python_stdlib("os"));
        assert!(is_python_stdlib("sys"));
        assert!(is_python_stdlib("json"));
        assert!(is_python_stdlib("typing"));
        assert!(is_python_stdlib("collections"));
        assert!(!is_python_stdlib("requests"));
        assert!(!is_python_stdlib("django"));
        assert!(!is_python_stdlib("numpy"));
    }

    #[test]
    fn classifies_modules_correctly() {
        assert_eq!(classify_module("os"), ImportType::Stdlib);
        assert_eq!(classify_module("sys"), ImportType::Stdlib);
        assert_eq!(classify_module("json"), ImportType::Stdlib);
        assert_eq!(classify_module("requests"), ImportType::ThirdParty);
        assert_eq!(classify_module("django.conf"), ImportType::ThirdParty);
        assert_eq!(classify_module(".config"), ImportType::Local);
        assert_eq!(classify_module("..utils"), ImportType::Local);
    }
}
