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
