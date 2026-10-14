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

fn man_rules(src: &str) -> Vec<String> {
    use dictator_decree_abi::Decree;

    dictator_freebsd::FreeBsdDecree
        .lint("probe.4", src)
        .iter()
        .map(|d| d.rule.clone())
        .collect()
}

#[test]
fn driver_page_nits_fixture() {
    let src = sandbox("driver_page_nits.4");
    let rules = man_rules(&src);

    for rule in [
        "freebsd/man-leading-blank-comment",
        "freebsd/man-synopsis-cd-only",
        "freebsd/man-hardware-list-bullet",
        "freebsd/man-macro-attached-period",
        "freebsd/man-lexicon",
    ] {
        assert!(
            rules.iter().any(|r| r == rule),
            "driver_page_nits.4 should trigger {rule}; got {rules:?}"
        );
    }
}

#[test]
fn clean_driver_page_is_quiet() {
    let rules = man_rules(&sandbox("clean_driver_page.4"));
    assert!(
        rules.is_empty(),
        "clean driver page should be quiet; got {rules:?}"
    );
}

#[test]
fn kernel_interface_synopsis_is_exempt() {
    // A SYNOPSIS documenting a function prototype alongside the config line
    // needs its .In/.Ft/.Fn lines, so the .Cd-only rule must stand down.
    let src = ".\\\"\n.Dd August 16, 2026\n.Dt PROBE 4\n.Os\n.Sh NAME\n.Nm probe\n\
               .Nd probe driver\n.Sh SYNOPSIS\n.Cd device probe\n.In sys/probe.h\n\
               .Ft int\n.Fn probe_attach \"device_t dev\"\n.Sh DESCRIPTION\nText.\n";
    let rules = man_rules(src);
    assert!(
        !rules.iter().any(|r| r == "freebsd/man-synopsis-cd-only"),
        "interface SYNOPSIS must not trigger man-synopsis-cd-only; got {rules:?}"
    );
}

#[test]
fn abbreviation_keeps_its_period() {
    let src = ".\\\"\n.Dd August 16, 2026\n.Dt PROBE 4\n.Os\n.Sh NAME\n.Nm probe\n\
               .Nd probe driver\n.Sh DESCRIPTION\n.Pq e.g.\n.Pp\nBuilt by Widgets Inc.\n";
    let rules = man_rules(src);
    assert!(
        !rules
            .iter()
            .any(|r| r == "freebsd/man-macro-attached-period"),
        "abbreviations must not trigger man-macro-attached-period; got {rules:?}"
    );
}
