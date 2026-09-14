    use super::*;

    #[test]
    fn test_insufficient_joy_zero_emojis() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_insufficient_joy("fn main() { println!(\"hello\"); }", &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/insufficient-joy"));
    }

    #[test]
    fn test_insufficient_joy_one_emoji() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        // One emoji (party popper U+1F389) - not enough
        check_insufficient_joy("// \u{1F389}\nfn main() {}", &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/insufficient-joy"));
    }

    #[test]
    fn test_sufficient_joy() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        // Two emojis (party popper + rocket) - sufficient
        check_insufficient_joy(
            "// \u{1F389}\u{1F680} Glory to Kim!\nfn main() {}",
            &config,
            &mut diags,
        );
        assert!(!diags.iter().any(|d| d.rule == "kjr/insufficient-joy"));
    }

    #[test]
    fn test_missing_dear_leader() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_missing_dear_leader("fn main() { }", &config, &mut diags);
        assert!(diags
            .iter()
            .any(|d| d.rule == "kjr/missing-dear-leader-comment"));
    }

    #[test]
    fn test_has_dear_leader_praise() {
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_missing_dear_leader(
            "// Glory to the Supreme Dictator!\nfn main() {}",
            &config,
            &mut diags,
        );
        assert!(!diags
            .iter()
            .any(|d| d.rule == "kjr/missing-dear-leader-comment"));
    }
