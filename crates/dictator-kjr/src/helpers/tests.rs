    use super::*;

    #[test]
    fn test_emoji_counting() {
        // Use actual emoji codepoints: U+1F389 (party popper), U+1F680 (rocket)
        assert_eq!(count_emojis("hello \u{1F389} world \u{1F680}"), 2);
        assert_eq!(count_emojis("no emojis here"), 0);
        assert_eq!(count_emojis("\u{1F389}\u{1F389}\u{1F389}"), 3);
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file("src/test_utils.rs"));
        assert!(is_test_file("spec/models/user_spec.rb"));
        assert!(is_test_file("main_test.go"));
        assert!(!is_test_file("src/main.rs"));
    }

    #[test]
    fn test_is_readme_file() {
        assert!(is_readme_file("README.md"));
        assert!(is_readme_file("readme.txt"));
        assert!(!is_readme_file("CHANGELOG.md"));
    }
