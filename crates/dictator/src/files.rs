//! File collection and type detection utilities

use anyhow::Result;
use camino::Utf8PathBuf;
use std::fs;
use walkdir::WalkDir;

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
            files.push(path.clone());
            continue;
        }

        for entry in WalkDir::new(path) {
            let entry = entry?;
            if !entry.file_type().is_file() {
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
            Some("ts" | "tsx" | "js" | "jsx") => types.has_typescript = true,
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
