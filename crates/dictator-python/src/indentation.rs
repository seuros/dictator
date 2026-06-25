//! Indentation consistency checks for Python sources.

use dictator_decree_abi::Diagnostics;

pub fn check_indentation_consistency(source: &str, diags: &mut Diagnostics) {
    dictator_supreme::check_indentation_consistency(source, "python", diags);
}
