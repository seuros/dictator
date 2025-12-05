//! File collection and type detection utilities

use anyhow::Result;
use camino::Utf8PathBuf;
use ignore::WalkBuilder;
use std::fs;

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

pub fn collect_all_files(paths: &[Utf8PathBuf]) -> Result<Vec<Utf8PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        let metadata = fs::metadata(path)?;
        if metadata.is_file() {
            // Explicit file path: always include, regardless of gitignore
            files.push(path.clone());
            continue;
        }

        // Directory: walk with gitignore filtering
        let walker = WalkBuilder::new(path)
            .standard_filters(true) // enables gitignore, .git/info/exclude, global config
            .build();

        for result in walker {
            let entry = result?;
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            let p = Utf8PathBuf::from_path_buf(entry.path().to_owned())
                .map_err(|_| anyhow::anyhow!("non-utf8 path: {}", entry.path().display()))?;
            files.push(p);
        }
    }
    Ok(files)
}

pub fn detect_file_types(files: &[Utf8PathBuf]) -> FileTypes {
    let mut types = FileTypes::default();
    for file in files {
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
