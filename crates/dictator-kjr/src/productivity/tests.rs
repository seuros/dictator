    use super::*;

    #[test]
    fn test_empty_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }

    #[test]
    fn test_whitespace_only_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("   \n\n   \t\n", &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }

    #[test]
    fn test_non_empty_file() {
        let mut diags = Diagnostics::new();
        check_empty_file("fn main() {}", &mut diags);
        assert!(!diags.iter().any(|d| d.rule == "kjr/empty-file"));
    }
