use super::*;

#[test]
fn valid_frontmatter_order() {
    // Default order is: title, description, pubDate
    let src = "---\ntitle: Test\ndescription: A description\npubDate: 2024-01-01\n---\n# Content\n";
    let diags = lint_source(src, "test.md");
    assert!(
        diags.is_empty(),
        "Expected no diagnostics for valid frontmatter"
    );
}

#[test]
fn detects_wrong_field_order() {
    // Default order is: title, description, pubDate
    // This has pubDate before title - wrong order
    let src = "---\npubDate: 2024-01-01\ndescription: Test desc\ntitle: Test\n---\n# Content\n";
    let diags = lint_source(src, "test.md");
    assert!(
        !diags.is_empty(),
        "Expected diagnostics for wrong field order"
    );
    assert_eq!(diags[0].rule, "decree.frontmatter/field-order");
}

#[test]
fn detects_missing_required_fields() {
    // Use custom config that requires both title and slug
    let config = FrontmatterConfig {
        order: vec!["title".to_string(), "slug".to_string()],
        required: vec!["title".to_string(), "slug".to_string()],
    };
    let src = "---\ntitle: Test\n---\n# Content\n";
    let diags = lint_source_with_config(src, "test.md", &config);
    assert!(
        !diags.is_empty(),
        "Expected diagnostics for missing required field"
    );
    let has_missing_slug = diags.iter().any(|d| {
        d.rule == "decree.frontmatter/missing-required-field" && d.message.contains("slug")
    });
    assert!(has_missing_slug);
}

#[test]
fn respects_custom_config() {
    // Custom config with different field order
    let config = FrontmatterConfig {
        order: vec![
            "title".to_string(),
            "description".to_string(),
            "pubDate".to_string(),
            "author".to_string(),
        ],
        required: vec!["title".to_string(), "description".to_string()],
    };

    // Valid order per custom config
    let src = "---\ntitle: Test\ndescription: A test\npubDate: 2024-01-01\n---\n# Content\n";
    let diags = lint_source_with_config(src, "test.md", &config);
    assert!(
        diags.is_empty(),
        "Expected no errors for valid custom order"
    );

    // Wrong order per custom config
    let src_wrong = "---\npubDate: 2024-01-01\ntitle: Test\n---\n# Content\n";
    let diags_wrong = lint_source_with_config(src_wrong, "test.md", &config);
    assert!(
        diags_wrong
            .iter()
            .any(|d| d.rule == "decree.frontmatter/field-order"),
        "Expected field order violation"
    );

    // Missing required field
    let src_missing = "---\ntitle: Test\n---\n# Content\n";
    let diags_missing = lint_source_with_config(src_missing, "test.md", &config);
    assert!(
        diags_missing
            .iter()
            .any(|d| d.rule == "decree.frontmatter/missing-required-field"
                && d.message.contains("description")),
        "Expected missing description error"
    );
}

#[test]
fn ignores_non_markdown_files() {
    let src = "title: Test\nslug: test\n";
    let diags = lint_source(src, "test.txt");
    assert!(diags.is_empty());
}

#[test]
fn supports_mdx_files() {
    let src = "---\ntitle: Test\nslug: test-slug\npubDate: 2024-01-01\n---\n\n\
                   import Component from './Component';\n\n# Content\n";
    let diags = lint_source(src, "test.mdx");
    assert!(
        diags.is_empty(),
        "Expected no diagnostics for valid MDX frontmatter"
    );
}

#[test]
fn ignores_yaml_files() {
    // YAML files are NOT frontmatter - they're standalone config files
    let src = "---\ntitle: Test\nslug: test\n---\n";
    let diags = lint_source(src, "config.yml");
    assert!(
        diags.is_empty(),
        "decree.frontmatter should not lint .yml files"
    );
}

#[test]
fn ignores_toml_files() {
    // TOML files are NOT frontmatter
    let src = "[package]\nname = \"test\"\n";
    let diags = lint_source(src, "Cargo.toml");
    assert!(
        diags.is_empty(),
        "decree.frontmatter should not lint .toml files"
    );
}

#[test]
fn ignores_astro_files() {
    // Astro files have JS/TS frontmatter, not YAML
    let src = "---\nconst title = 'Test';\n---\n<html>{title}</html>\n";
    let diags = lint_source(src, "page.astro");
    assert!(
        diags.is_empty(),
        "decree.frontmatter should not lint .astro files"
    );
}

#[test]
fn handles_markdown_without_frontmatter() {
    let src = "# Content\nNo frontmatter here\n";
    let diags = lint_source(src, "test.md");
    assert!(diags.is_empty());
}

#[test]
fn detects_invalid_yaml() {
    let src = "---\ntitle: [broken yaml\n---\n# Content\n";
    let diags = lint_source(src, "test.md");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].rule, "decree.frontmatter/invalid-yaml");
}

#[test]
fn sandbox_blog_wrong_order() {
    // Test the actual sandbox file case: pubDate comes before title
    // Default order: title, description, pubDate
    // This frontmatter has: pubDate, description, title (wrong!)
    let src = "---\npubDate: 2024-12-01\n\
                   description: This blog post has wrong frontmatter ordering\n\
                   title: Blog Post With Wrong Frontmatter Order\n\
                   author: John Doe\n---\n\n# Blog Post Content\n";
    let diags = lint_source(src, "blog-wrong-frontmatter-order.md");
    assert!(
        !diags.is_empty(),
        "Expected to detect field order violation"
    );

    assert!(
        diags
            .iter()
            .any(|d| d.rule == "decree.frontmatter/field-order"),
        "Expected field order violation diagnostic"
    );
}
