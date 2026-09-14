//! Integration test over `sandbox/freebsd/` — bad C fixtures that should make
//! the freebsd decree scream. No per-rule assertions; the fixture just has to
//! produce diagnostics.

use dictator_freebsd::lint_source;

fn sandbox(rel: &str) -> String {
    let path = format!("{}/../../sandbox/freebsd/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("sandbox fixture {rel}: {e}"))
}

#[test]
fn style9_violations_fixture() {
    let diags = lint_source(&sandbox("style9_violations.c"));
    assert!(
        !diags.is_empty(),
        "style9_violations.c should trigger freebsd style(9) diagnostics"
    );
}

#[test]
fn bad_man_page_fixture() {
    use dictator_decree_abi::Decree;

    let src = sandbox("bad_man_page.8");
    let diags = dictator_freebsd::FreeBsdDecree.lint("bad_man_page.8", &src);
    assert!(
        !diags.is_empty(),
        "bad_man_page.8 should trigger freebsd mdoc(7)/style.mdoc(5) diagnostics"
    );
}
