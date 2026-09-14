mod support;
use support::*;

#[test]
fn free_flagged() {
    assert_flags("\tfree(ptr);\n", "free-is-noop");
}

#[test]
fn identifier_ending_in_free_not_flagged() {
    assert_clean("\tbuffer_free(ptr);\n");
}
