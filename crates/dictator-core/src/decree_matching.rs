//! File-to-decree matching logic.

use camino::Utf8Path;
use dictator_decree_abi::BoxDecree;

/// Language-specific decrees that shadow decree.supreme.
const SHADOWERS: [&str; 6] = ["ruby", "typescript", "golang", "rust", "python", "freebsd"];

/// Check if a decree matches a file (by extension or filename).
pub fn decree_matches(path: &Utf8Path, meta: &dictator_decree_abi::DecreeMetadata) -> bool {
    let filename = path.file_name().unwrap_or("");

    // Universal decree (empty lists) matches everything
    if meta.supported_extensions.is_empty() && meta.supported_filenames.is_empty() {
        return true;
    }

    // Check filename match
    if meta.supported_filenames.iter().any(|s| s == filename) {
        return true;
    }

    // Check extension match
    extension_matches(path, &meta.supported_extensions)
}

/// Check if a file's extension matches any in the supported list.
pub fn extension_matches(path: &Utf8Path, supported: &[String]) -> bool {
    path.extension()
        .is_some_and(|ext| supported.iter().any(|s| s == ext))
}

/// Check if supreme should be shadowed for this path.
///
/// Only language-specific decrees shadow decree.supreme. Other decrees (e.g. frontmatter
/// or custom plugins) remain additive and run alongside supreme.
pub fn is_supreme_shadowed(decrees: &[BoxDecree], path: &Utf8Path) -> bool {
    decrees.iter().any(|decree| {
        let name = decree.name();
        if !SHADOWERS.contains(&name) {
            return false;
        }

        let meta = decree.metadata();

        // Check if this shadower handles this file
        decree_matches(path, &meta)
    })
}

#[cfg(test)]
mod tests;
