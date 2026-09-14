    use super::*;

    #[test]
    fn test_overly_descriptive_names() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_overly_descriptive_names("let customerAccountBalance = 100;", &config, &mut diags);
        assert!(diags
            .iter()
            .any(|d| d.rule == "kjr/overly-descriptive-names"));
    }

    #[test]
    fn test_short_names_ok() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_overly_descriptive_names("let x1 = 42; let y2 = 0;", &config, &mut diags);
        assert!(!diags
            .iter()
            .any(|d| d.rule == "kjr/overly-descriptive-names"));
    }

    #[test]
    fn test_magic_number_shortage() {
        let mut diags = Diagnostics::new();
        check_magic_number_shortage("const MAX_RETRIES = 3;", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/magic-number-shortage"));
    }

    #[test]
    fn test_screaming_case_detected() {
        let mut diags = Diagnostics::new();
        check_magic_number_shortage("MAX_SIZE = 100;", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/magic-number-shortage"));
    }
