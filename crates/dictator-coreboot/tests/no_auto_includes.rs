mod support;
use support::*;

#[test]
fn kconfig_include_flagged() {
    assert_flags("#include <kconfig.h>\n", "no-auto-includes");
}

#[test]
fn config_include_flagged() {
    assert_flags("#include <config.h>\n", "no-auto-includes");
}

#[test]
fn rules_include_flagged() {
    assert_flags("#include <rules.h>\n", "no-auto-includes");
}

#[test]
fn compiler_include_flagged_through_path() {
    // matches both <compiler.h> and <commonlib/bsd/compiler.h>
    assert_flags("#include <commonlib/bsd/compiler.h>\n", "no-auto-includes");
}

#[test]
fn normal_include_ok() {
    assert_clean("#include <stdint.h>\n");
    assert_clean("#include <device/pci.h>\n");
}
