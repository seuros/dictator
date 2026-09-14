//! Integration tests over `sandbox/configs/` blog fixtures
//! (see `sandbox/EXPECTED_VIOLATIONS.md`, decree.frontmatter section).

use dictator_frontmatter::{FrontmatterConfig, lint_source, lint_source_with_config};

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/configs/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

fn blog_config() -> FrontmatterConfig {
    // The sandbox blog contract: title, slug, pubDate, description, tags.
    FrontmatterConfig {
        order: ["title", "slug", "pubDate", "description", "tags"].map(String::from).to_vec(),
        required: ["title", "slug"].map(String::from).to_vec(),
    }
}

#[test]
fn invalid_yaml_fixture() {
    let diags = lint_source(&sandbox("blog-invalid-yaml.md"), "blog-invalid-yaml.md");
    assert!(
        diags.iter().any(|d| d.rule == "decree.frontmatter/invalid-yaml"),
        "blog-invalid-yaml.md should trigger invalid-yaml"
    );
}

#[test]
fn wrong_field_order_fixture() {
    let diags = lint_source_with_config(
        &sandbox("blog-wrong-frontmatter-order.md"),
        "blog-wrong-frontmatter-order.md",
        &blog_config(),
    );
    assert!(
        diags.iter().any(|d| d.rule == "decree.frontmatter/field-order"),
        "blog-wrong-frontmatter-order.md should trigger field-order"
    );
}

#[test]
fn missing_required_field_fixture() {
    let diags = lint_source_with_config(
        &sandbox("blog-missing-required-field.md"),
        "blog-missing-required-field.md",
        &blog_config(),
    );
    assert!(
        diags.iter().any(|d| d.rule == "decree.frontmatter/missing-required-field"),
        "blog-missing-required-field.md should trigger missing-required-field (slug)"
    );
}

#[test]
fn valid_fixtures_stay_clean() {
    for rel in ["blog-valid-frontmatter.md", "blog-no-frontmatter.md"] {
        let diags = lint_source_with_config(&sandbox(rel), rel, &blog_config());
        assert!(diags.is_empty(), "{rel} should be violation-free, got: {:?}", diags);
    }
}
