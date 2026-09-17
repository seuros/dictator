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
fn c99_violations_fixture() {
    let diags = lint_source(&sandbox("c99_violations.c"));

    for rule in [
        "freebsd/c99-obsolete-storage-class",
        "freebsd/c99-empty-param-list",
        "freebsd/c99-knr-definition",
        "freebsd/c99-legacy-int-types",
        "freebsd/c99-bool-macros",
        "freebsd/c99-gnu-inline",
        "freebsd/c99-named-variadic-macro",
    ] {
        assert!(
            diags.iter().any(|d| d.rule == rule),
            "c99_violations.c should trigger {rule}; got {:?}",
            diags.iter().map(|d| &d.rule).collect::<Vec<_>>()
        );
    }
}

#[test]
fn c99_clean_source_is_quiet() {
    let clean = "\
static inline uint32_t\nmask_of(uint32_t v)\n{\n\n\treturn (v);\n}\n";
    let diags = lint_source(clean);
    assert!(
        !diags.iter().any(|d| d.rule.contains("/c99-")),
        "prototyped C99 source should not trigger c99 rules; got {:?}",
        diags.iter().map(|d| &d.rule).collect::<Vec<_>>()
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
