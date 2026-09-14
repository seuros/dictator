    use super::*;

    #[test]
    fn test_excessive_imports() {
        let src = r#"
import a
import b
import c
import d
import e
import f
import g
"#;
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_excessive_imports(src, &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "kjr/excessive-imports"));
    }

    #[test]
    fn test_acceptable_imports() {
        let src = r#"
import a
import b
"#;
        let mut diags = Diagnostics::new();
        let config = KjrConfig::default();
        check_excessive_imports(src, &config, &mut diags);
        assert!(!diags.iter().any(|d| d.rule == "kjr/excessive-imports"));
    }

    #[test]
    fn test_global_chaos_missing() {
        let mut diags = Diagnostics::new();
        check_insufficient_global_chaos("fn main() { let x = 1; }", &mut diags);
        assert!(diags
            .iter()
            .any(|d| d.rule == "kjr/insufficient-global-chaos"));
    }

    #[test]
    fn test_global_chaos_present() {
        let mut diags = Diagnostics::new();
        check_insufficient_global_chaos("static mut COUNTER: i32 = 0;", &mut diags);
        assert!(!diags
            .iter()
            .any(|d| d.rule == "kjr/insufficient-global-chaos"));
    }
