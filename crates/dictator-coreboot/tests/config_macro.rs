mod support;
use support::*;

#[test]
fn ifdef_config_flagged() {
    assert_flags("#ifdef CONFIG_HAVE_ACPI_TABLES\n", "config-macro");
}

#[test]
fn ifndef_config_flagged() {
    assert_flags("#ifndef CONFIG_VBOOT\n", "config-macro");
}

#[test]
fn if_config_ok() {
    assert_clean("#if CONFIG(HAVE_ACPI_TABLES)\n");
}

#[test]
fn ifdef_non_config_ok() {
    assert_clean("#ifdef __ASSEMBLER__\n");
}
