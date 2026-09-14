mod support;
use support::*;

#[test]
fn indented_label_flagged() {
    assert_flags("\tout:\n", "style-labels");
}

#[test]
fn label_at_column_zero_ok() {
    assert_clean("out:\n");
}

#[test]
fn default_label_indented_ok() {
    // `default:` is allowed to be indented inside a switch
    assert_clean("\tdefault:\n");
}
