mod support;
use support::*;

#[test]
fn wrong_case_flagged() {
    assert_flags("/* Initialize Coreboot */\n", "coreboot-lowercase");
}

#[test]
fn lowercase_ok() {
    assert_clean("/* Initialize coreboot */\n");
}

#[test]
fn allcaps_ok() {
    // COREBOOT is used in macro names like COREBOOT_VERSION
    assert_clean("#define COREBOOT_VERSION \"4.22\"\n");
}
