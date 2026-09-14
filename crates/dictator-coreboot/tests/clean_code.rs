//! Compliant code must not trip any coreboot decree.

mod support;
use support::*;

#[test]
fn clean_function_no_diags() {
    assert_clean(
        "static void setup_device(struct device *dev)\n\
         {\n\
         \tprintk(BIOS_DEBUG, \"setting up device\\n\");\n\
         \tif (CONFIG(HAVE_ACPI_TABLES))\n\
         \t\tdo_acpi();\n\
         }\n",
    );
}
