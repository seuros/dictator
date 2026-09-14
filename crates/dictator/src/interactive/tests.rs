use super::*;

#[test]
fn test_interactive_fixer_creation() {
    let fixer = InteractiveFixer::new();
    assert_eq!(fixer.violations.len(), 0);
    assert_eq!(fixer.current_index, 0);
    assert!(!fixer.auto_apply_all);
}
