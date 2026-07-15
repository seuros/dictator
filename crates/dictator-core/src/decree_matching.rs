//! File-to-decree matching logic.

use camino::Utf8Path;
use dictator_decree_abi::BoxDecree;

/// Language-specific decrees that shadow decree.supreme.
const SHADOWERS: [&str; 5] = ["ruby", "typescript", "golang", "rust", "python"];

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
    path.extension().is_some_and(|ext| supported.iter().any(|s| s == ext))
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
mod tests {
    use super::*;
    use dictator_decree_abi::{Capability, Decree, DecreeMetadata, Diagnostics};
    use dictator_decree_abi::{Diagnostic, Span};

    struct MockDecree {
        name: &'static str,
        exts: Vec<String>,
        filenames: Vec<String>,
        rule: &'static str,
    }

    impl MockDecree {
        fn simple(name: &'static str, exts: Vec<String>, rule: &'static str) -> Self {
            Self { name, exts, filenames: vec![], rule }
        }
    }

    impl Decree for MockDecree {
        fn name(&self) -> &str {
            self.name
        }

        fn lint(&self, _path: &str, _source: &str) -> Diagnostics {
            vec![Diagnostic {
                rule: self.rule.to_string(),
                message: format!("hit {}", self.name),
                span: Span::new(0, 0),
                enforced: false,
            }]
        }

        fn metadata(&self) -> DecreeMetadata {
            DecreeMetadata {
                abi_version: "1".into(),
                decree_version: "1".into(),
                description: String::new(),
                dectauthors: None,
                supported_extensions: self.exts.clone(),
                supported_filenames: self.filenames.clone(),
                skip_filenames: vec![],
                capabilities: vec![Capability::Lint],
            }
        }
    }

    #[test]
    fn supreme_shadowed_by_language_decree() {
        let supreme: BoxDecree = Box::new(MockDecree::simple("supreme", vec![], "supreme/hit"));
        let ruby: BoxDecree = Box::new(MockDecree::simple("ruby", vec!["rb".into()], "ruby/hit"));

        let decrees = vec![supreme, ruby];
        let path = Utf8Path::new("test.rb");

        assert!(
            is_supreme_shadowed(&decrees, path),
            "supreme should be shadowed for .rb files when ruby decree exists"
        );
    }

    #[test]
    fn supreme_not_shadowed_for_unmatched_extension() {
        let supreme: BoxDecree = Box::new(MockDecree::simple("supreme", vec![], "supreme/hit"));
        let ruby: BoxDecree = Box::new(MockDecree::simple("ruby", vec!["rb".into()], "ruby/hit"));

        let decrees = vec![supreme, ruby];
        let path = Utf8Path::new("test.txt");

        assert!(
            !is_supreme_shadowed(&decrees, path),
            "supreme should not be shadowed for .txt files"
        );
    }

    #[test]
    fn non_language_decree_does_not_shadow() {
        let supreme: BoxDecree = Box::new(MockDecree::simple("supreme", vec![], "supreme/hit"));
        let frontmatter: BoxDecree =
            Box::new(MockDecree::simple("frontmatter", vec!["md".into()], "frontmatter/hit"));

        let decrees = vec![supreme, frontmatter];
        let path = Utf8Path::new("README.md");

        assert!(!is_supreme_shadowed(&decrees, path), "frontmatter should not shadow supreme");
    }
}
