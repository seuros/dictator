mod support;
use support::*;

#[test]
fn function_macro_flagged() {
    assert_flags(
        "\tprintk(BIOS_ERR, \"%s: fail\\n\", __FUNCTION__);\n",
        "function-name",
    );
}

#[test]
fn func_ok() {
    assert_clean("\tprintk(BIOS_ERR, \"%s\\n\", __func__);\n");
}
