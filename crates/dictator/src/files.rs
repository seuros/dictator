//! File collection and type detection utilities

use anyhow::Result;
use camino::Utf8PathBuf;
use ignore::WalkBuilder;

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct FileTypes {
    pub has_ruby: bool,
    pub has_typescript: bool,
    pub has_golang: bool,
    pub has_rust: bool,
    pub has_python: bool,
    pub has_configs: bool,
}

pub fn read_source_file(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Walk a single path and collect source files.
///
/// Uses `ignore::WalkBuilder` with:
/// - `standard_filters(true)` — respects `.gitignore`, `.ignore`, hidden files
/// - `require_git(false)` — honours `.gitignore` files even outside git repos
///
/// This is the single walker for both CLI and MCP paths.
pub fn collect_source_files(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.to_path_buf());
        return files;
    }

    let walker = WalkBuilder::new(path)
        .standard_filters(true)
        .require_git(false)
        .build();

    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            files.push(entry.into_path());
        }
    }
    files
}

pub fn collect_all_files(paths: &[Utf8PathBuf]) -> Result<Vec<Utf8PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        for p in collect_source_files(path.as_std_path()) {
            let utf8 = Utf8PathBuf::from_path_buf(p)
                .map_err(|p| anyhow::anyhow!("non-utf8 path: {}", p.display()))?;
            files.push(utf8);
        }
    }
    Ok(files)
}

pub fn detect_file_types(files: &[Utf8PathBuf]) -> FileTypes {
    let mut types = FileTypes::default();
    for file in files {
        // Check by filename first (for files like Cargo.toml, Gemfile, etc.)
        if let Some(filename) = file.file_name() {
            match filename {
                // Rust-specific filenames
                "Cargo.toml"
                | "build.rs"
                | "rust-toolchain"
                | "rust-toolchain.toml"
                | ".rustfmt.toml"
                | "rustfmt.toml"
                | "clippy.toml"
                | ".clippy.toml" => {
                    types.has_rust = true;
                    continue;
                }
                // Ruby-specific filenames
                "Gemfile" | "Rakefile" | ".rubocop.yml" | ".ruby-version" => {
                    types.has_ruby = true;
                    continue;
                }
                // Python-specific filenames
                "pyproject.toml" | "setup.py" | "requirements.txt" | ".python-version" => {
                    types.has_python = true;
                    continue;
                }
                // Go-specific filenames
                "go.mod" | "go.sum" => {
                    types.has_golang = true;
                    continue;
                }
                // TypeScript-specific filenames
                "tsconfig.json" | "package.json" | "biome.json" | "biome.jsonc"
                | ".eslintrc.json" => {
                    types.has_typescript = true;
                    continue;
                }
                _ => {}
            }
        }

        // Then check by extension
        match file.extension() {
            Some("rb" | "rake") => types.has_ruby = true,
            Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs") => types.has_typescript = true,
            Some("go") => types.has_golang = true,
            Some("rs") => types.has_rust = true,
            Some("py") => types.has_python = true,
            // Frontmatter only applies to .md and .mdx (YAML frontmatter)
            // .astro has JS/TS frontmatter - not handled by decree.frontmatter
            // .yml/.yaml/.toml are standalone config files - need separate decrees
            Some("md" | "mdx") => types.has_configs = true,
            _ => {}
        }
    }
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> Utf8PathBuf {
        Utf8PathBuf::from(name)
    }

    #[test]
    fn detects_mjs_and_cjs_as_typescript() {
        let files = vec![path("web/app.mjs"), path("cli/tools.cjs")];
        let types = detect_file_types(&files);
        assert!(types.has_typescript);
        assert!(!types.has_ruby && !types.has_golang && !types.has_rust && !types.has_python);
    }

    #[test]
    fn detects_markdown_as_config() {
        let files = vec![path("content/post.md"), path("content/page.mdx")];
        let types = detect_file_types(&files);
        assert!(types.has_configs);
    }
}
