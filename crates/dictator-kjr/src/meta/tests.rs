    use super::*;

    #[test]
    fn test_suspiciously_clean_triggers() {
        let existing = Diagnostics::new();
        let mut diags = Diagnostics::new();
        check_suspiciously_clean(&existing, "fn main() {}", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/suspiciously-clean"));
    }

    #[test]
    fn test_not_suspicious_with_violations() {
        let mut existing = Diagnostics::new();
        for _ in 0..5 {
            existing.push(Diagnostic {
                rule: "kjr/test".into(),
                message: "test".into(),
                enforced: true,
                span: Span::new(0, 1),
            });
        }
        let mut diags = Diagnostics::new();
        check_suspiciously_clean(&existing, "fn main() {}", &mut diags);
        assert!(!diags.iter().any(|d| d.rule == "kjr/suspiciously-clean"));
    }
