#![warn(rust_2024_compatibility, clippy::all)]

//! decree.typescript - TypeScript/JavaScript structural rules.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use dictator_supreme::SupremeConfig;
use memchr::memchr_iter;

/// Lint TypeScript source for structural violations.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_configs(source, &TypeScriptConfig::default(), &SupremeConfig::default())
}

/// Lint TypeScript source with custom configuration.
#[must_use]
pub fn lint_source_with_config(source: &str, config: &TypeScriptConfig) -> Diagnostics {
    lint_source_with_configs(source, config, &SupremeConfig::default())
}

/// Lint TypeScript source with custom config + supreme config.
#[must_use]
pub fn lint_source_with_configs(
    source: &str,
    config: &TypeScriptConfig,
    supreme_config: &SupremeConfig,
) -> Diagnostics {
    let mut diags = Diagnostics::new();

    let supreme_diags =
        dictator_supreme::lint_source_with_owner(source, supreme_config, "typescript");

    if config.ignore_comments {
        diags.extend(dictator_supreme::retain_long_line_diags(
            source,
            supreme_diags,
            "typescript/line-too-long",
            "//",
        ));
    } else {
        diags.extend(supreme_diags);
    }

    // TypeScript-specific rules
    check_file_line_count(source, config.max_lines, &mut diags);
    check_import_ordering(source, &mut diags);
    check_indentation_consistency(source, &mut diags);

    diags
}

/// Configuration for typescript decree
#[derive(Debug, Clone)]
pub struct TypeScriptConfig {
    pub max_lines: usize,
    pub ignore_comments: bool,
}

impl Default for TypeScriptConfig {
    fn default() -> Self {
        Self { max_lines: 350, ignore_comments: false }
    }
}

/// Rule 1: File line count - configurable max lines (ignoring comments and blank lines)
fn check_file_line_count(source: &str, max_lines: usize, diags: &mut Diagnostics) {
    let code_lines = dictator_supreme::count_code_lines(source, is_comment_only_line);

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "typescript/file-too-long".to_string(),
            message: format!(
                "File has {code_lines} code lines \
                 (max {max_lines}, excluding comments and blank lines)"
            ),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// Check if a line is comment-only (// or /* */ style)
fn is_comment_only_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*')
}

/// Rule 2: Import ordering - system → external → internal
fn check_import_ordering(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut imports: Vec<(usize, usize, ImportType)> = Vec::new();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        if let Some(import_type) = parse_import_line(trimmed) {
            imports.push((line_start, nl, import_type));
        }

        // Stop at first non-import, non-comment, non-blank line
        if !trimmed.is_empty()
            && !trimmed.starts_with("import")
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
            && !trimmed.starts_with('*')
            && !trimmed.starts_with("export")
        {
            break;
        }

        line_start = nl + 1;
    }

    // Check import order
    if imports.len() > 1 {
        let mut last_type = ImportType::System;

        for (start, end, import_type) in &imports {
            // Order should be: System → External → Internal
            let type_order = match import_type {
                ImportType::System => 0,
                ImportType::External => 1,
                ImportType::Internal => 2,
            };

            let last_type_order = match last_type {
                ImportType::System => 0,
                ImportType::External => 1,
                ImportType::Internal => 2,
            };

            if type_order < last_type_order {
                diags.push(Diagnostic {
                    rule: "typescript/import-order".to_string(),
                    message: format!(
                        "Import order violation: {import_type:?} import after \
                         {last_type:?} import. Expected: system → external → internal"
                    ),
                    enforced: false,
                    span: Span::new(*start, *end),
                });
            }

            last_type = *import_type;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportType {
    System,   // Node.js built-ins: fs, path, crypto, events, etc.
    External, // npm packages
    Internal, // Relative imports: ./ or ../
}

/// Parse an import line and determine its type
fn parse_import_line(line: &str) -> Option<ImportType> {
    if !line.starts_with("import") {
        return None;
    }

    // Extract the module name from import statement
    // Patterns: import ... from 'module' or import ... from "module"
    let from_pos = line.find(" from ")?;
    let after_from = &line[from_pos + 6..].trim();

    // Extract quoted module name
    let quote_start = after_from.find(['\'', '"'])?;
    let quote_char = after_from.chars().nth(quote_start)?;
    let module_start = quote_start + 1;
    let module_end = after_from[module_start..].find(quote_char)?;
    let module_name = &after_from[module_start..module_start + module_end];

    // Determine type
    if module_name.starts_with('.') {
        Some(ImportType::Internal)
    } else if is_nodejs_builtin(module_name) {
        Some(ImportType::System)
    } else {
        Some(ImportType::External)
    }
}

/// Check if module is a Node.js built-in
fn is_nodejs_builtin(module: &str) -> bool {
    // Remove 'node:' prefix if present
    let module = module.strip_prefix("node:").unwrap_or(module);

    matches!(
        module,
        "fs" | "path"
            | "crypto"
            | "events"
            | "http"
            | "https"
            | "os"
            | "util"
            | "url"
            | "stream"
            | "buffer"
            | "child_process"
            | "cluster"
            | "dns"
            | "net"
            | "readline"
            | "repl"
            | "tls"
            | "dgram"
            | "zlib"
            | "querystring"
            | "string_decoder"
            | "timers"
            | "tty"
            | "vm"
            | "assert"
            | "console"
            | "process"
            | "v8"
            | "perf_hooks"
            | "worker_threads"
            | "async_hooks"
    )
}

/// Rule 3: Indentation consistency
fn check_indentation_consistency(source: &str, diags: &mut Diagnostics) {
    dictator_supreme::check_indentation_consistency(source, "typescript", diags);
}

#[derive(Default)]
pub struct TypeScript {
    config: TypeScriptConfig,
    supreme: SupremeConfig,
}

impl TypeScript {
    #[must_use]
    pub const fn new(config: TypeScriptConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for TypeScript {
    fn name(&self) -> &'static str {
        "typescript"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        lint_source_with_configs(source, &self.config, &self.supreme)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "TypeScript/JavaScript structural rules".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "mjs".to_string(),
                "cjs".to_string(),
                "mts".to_string(),
                "cts".to_string(),
            ],
            supported_filenames: vec![
                "package.json".to_string(),
                "tsconfig.json".to_string(),
                "jsconfig.json".to_string(),
                "biome.json".to_string(),
                "biome.jsonc".to_string(),
                ".eslintrc".to_string(),
                ".prettierrc".to_string(),
                "deno.json".to_string(),
                "deno.jsonc".to_string(),
                "bunfig.toml".to_string(),
            ],
            skip_filenames: vec![
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
                "bun.lockb".to_string(),
                "deno.lock".to_string(),
                "npm-shrinkwrap.json".to_string(),
            ],
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(TypeScript::default())
}

/// Create plugin with custom config
#[must_use]
pub fn init_decree_with_config(config: TypeScriptConfig) -> BoxDecree {
    Box::new(TypeScript::new(config, SupremeConfig::default()))
}

/// Create plugin with custom config + supreme config
/// (merged from decree.supreme + decree.typescript)
#[must_use]
pub fn init_decree_with_configs(config: TypeScriptConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(TypeScript::new(config, supreme))
}

/// Convert `DecreeSettings` to `TypeScriptConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> TypeScriptConfig {
    TypeScriptConfig {
        max_lines: settings.max_lines.unwrap_or(350),
        ignore_comments: settings.ignore_comments.unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_file_too_long() {
        use std::fmt::Write;
        // Create a file with 400 code lines (excluding blank lines and comments)
        let mut src = String::new();
        for i in 0..400 {
            let _ = writeln!(src, "const x{i} = {i};");
        }
        let diags = lint_source(&src);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/file-too-long"),
            "Should detect file with >350 code lines"
        );
    }

    #[test]
    fn ignores_comments_in_line_count() {
        use std::fmt::Write;
        // 340 code lines + 60 comment lines = 400 total, but only 340 counted
        let mut src = String::new();
        for i in 0..340 {
            let _ = writeln!(src, "const x{i} = {i};");
        }
        for i in 0..60 {
            let _ = writeln!(src, "// Comment {i}");
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
            "Should not count comment-only lines"
        );
    }

    #[test]
    fn ignores_blank_lines_in_count() {
        use std::fmt::Write;
        // 340 code lines + 60 blank lines = 400 total, but only 340 counted
        let mut src = String::new();
        for i in 0..340 {
            let _ = writeln!(src, "const x{i} = {i};");
        }
        for _ in 0..60 {
            src.push('\n');
        }
        let diags = lint_source(&src);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
            "Should not count blank lines"
        );
    }

    #[test]
    fn detects_wrong_import_order_system_after_external() {
        let src = r"
import { format } from 'date-fns';
import * as fs from 'fs';
import { config } from './config';
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/import-order"),
            "Should detect system import after external import"
        );
    }

    #[test]
    fn detects_wrong_import_order_internal_before_external() {
        let src = r"
import { config } from './config';
import { format } from 'date-fns';
import * as fs from 'fs';
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/import-order"),
            "Should detect wrong import order"
        );
    }

    #[test]
    fn accepts_correct_import_order() {
        let src = r"
import * as fs from 'fs';
import * as path from 'path';
import { format } from 'date-fns';
import axios from 'axios';
import { config } from './config';
import type { Logger } from './types';
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/import-order"),
            "Should accept correct import order"
        );
    }

    #[test]
    fn detects_mixed_tabs_and_spaces() {
        let src = "function test() {\n\tconst x = 1;\n  const y = 2;\n}\n";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/mixed-indentation"),
            "Should detect mixed tabs and spaces"
        );
    }

    #[test]
    fn detects_inconsistent_indentation_depth() {
        let src = r"
function test() {
  if (true) {
     const x = 1;
  }
}
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/inconsistent-indentation"),
            "Should detect inconsistent indentation depth (3 spaces instead of 2 or 4)"
        );
    }

    #[test]
    fn accepts_consistent_indentation() {
        let src = r"
function test() {
  if (true) {
    const x = 1;
    const y = 2;
  }
}
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/mixed-indentation"
                || d.rule == "typescript/inconsistent-indentation"),
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
        let src = "// Comment 1\n// Comment 2\n/* Block comment */\n";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/file-too-long"),
            "File with only comments should not trigger line count"
        );
    }

    #[test]
    fn detects_nodejs_builtins_correctly() {
        assert!(is_nodejs_builtin("fs"));
        assert!(is_nodejs_builtin("path"));
        assert!(is_nodejs_builtin("crypto"));
        assert!(is_nodejs_builtin("events"));
        assert!(!is_nodejs_builtin("date-fns"));
        assert!(!is_nodejs_builtin("lodash"));
        assert!(!is_nodejs_builtin("./config"));
    }

    #[test]
    fn ignores_long_comment_lines_when_configured() {
        let long_comment = format!("// {}\n", "x".repeat(150));
        let src = format!("function foo() {{\n{long_comment}}}\n");
        let config = TypeScriptConfig { ignore_comments: true, ..Default::default() };
        let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(
            !diags.iter().any(|d| d.rule == "typescript/line-too-long"),
            "Should ignore long comment lines when ignore_comments is true"
        );
    }

    #[test]
    fn detects_long_comment_lines_when_not_configured() {
        let long_comment = format!("// {}\n", "x".repeat(150));
        let src = format!("function foo() {{\n{long_comment}}}\n");
        let config = TypeScriptConfig::default(); // ignore_comments = false
        let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/line-too-long"),
            "Should detect long comment lines when ignore_comments is false"
        );
    }

    #[test]
    fn still_detects_long_code_lines_with_ignore_comments() {
        let long_code = format!("  const x = \"{}\";\n", "a".repeat(150));
        let src = format!("function foo() {{\n{long_code}}}\n");
        let config = TypeScriptConfig { ignore_comments: true, ..Default::default() };
        let supreme = SupremeConfig { max_line_length: Some(120), ..Default::default() };
        let diags = lint_source_with_configs(&src, &config, &supreme);
        assert!(
            diags.iter().any(|d| d.rule == "typescript/line-too-long"),
            "Should still detect long code lines even with ignore_comments enabled"
        );
    }
}
