mod support;
use support::*;

#[test]
fn printf_flagged() {
    assert_flags("\tprintf(\"hello\\n\");\n", "no-printf");
}

#[test]
fn snprintf_not_flagged() {
    assert_clean("\tsnprintf(buf, sizeof(buf), \"%d\", n);\n");
}

#[test]
fn cbprintf_not_flagged() {
    assert_clean("\tcbprintf(putchar, \"%s\", str);\n");
}

#[test]
fn printf_in_macro_define_is_a_known_false_positive() {
    // device_tree.c pattern: #define printk(level, ...) printf(__VA_ARGS__)
    // The char before `printf` is a space, so find_bare_call flags it.
    // Accepted false positive in vendorcode; asserted here so a future fix is noticed.
    assert_flags(
        "#define printk(level, ...) printf(__VA_ARGS__)\n",
        "no-printf",
    );
}
