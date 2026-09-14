mod support;
use support::*;

#[test]
fn die_flagged() {
    assert_flags("\tdie(\"unrecoverable\\n\");\n", "die-usage");
}
