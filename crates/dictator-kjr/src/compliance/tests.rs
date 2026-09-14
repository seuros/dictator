    use super::*;

    #[test]
    fn test_no_todo() {
        let mut diags = Diagnostics::new();
        check_no_todo("// TODO: fix this\nfn main() {}", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/no-todo"));
    }

    #[test]
    fn test_fixme_detected() {
        let mut diags = Diagnostics::new();
        check_no_todo("// FIXME: broken\nfn main() {}", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/no-todo"));
    }

    #[test]
    fn test_capitalist_naming() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_capitalist_naming("let profit = calculate_revenue();", &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/capitalist-naming"));
    }

    #[test]
    fn test_singleton_detected() {
        let mut diags = Diagnostics::new();
        check_singleton_detected("def getInstance(): return _instance", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/singleton-detected"));
    }
