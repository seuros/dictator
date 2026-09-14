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

    assert!(!is_supreme_shadowed(&decrees, path), "supreme should not be shadowed for .txt files");
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
