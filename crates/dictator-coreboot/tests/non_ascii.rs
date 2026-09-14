mod support;
use support::*;

#[test]
fn non_ascii_flagged() {
    assert_flags("/* résumé */\n", "non-ascii");
}

#[test]
fn ascii_ok() {
    assert_clean("/* normal comment */\n");
}
