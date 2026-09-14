mod support;
use support::*;

#[test]
fn printk_without_bios_level_flagged() {
    assert_flags("\tprintk(\"hello\\n\");\n", "printk-log-level");
}

#[test]
fn printk_with_bios_debug_ok() {
    assert_clean("\tprintk(BIOS_DEBUG, \"val=%d\\n\", v);\n");
}

#[test]
fn all_bios_levels_ok() {
    for level in &[
        "BIOS_EMERG",
        "BIOS_ALERT",
        "BIOS_CRIT",
        "BIOS_ERR",
        "BIOS_WARNING",
        "BIOS_NOTICE",
        "BIOS_INFO",
        "BIOS_DEBUG",
        "BIOS_SPEW",
    ] {
        assert_clean(&format!("\tprintk({level}, \"msg\\n\");\n"));
    }
}
