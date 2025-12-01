//! Helper functions and precompiled regexes
//!
//! The tools of the revolution, optimized for maximum efficiency.

use regex::Regex;
use std::sync::LazyLock;

// =============================================================================
// PRECOMPILED REGEXES
// =============================================================================

pub static IDENTIFIER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]{12,})\b").unwrap());

pub static CONSTANT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(const|final|static)\s+[A-Z_]+\s*=").unwrap());

pub static SCREAMING_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Z][A-Z0-9_]{2,})\s*=").unwrap());

pub static IMPORT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(import|require|use|from|#include)\b").unwrap());

pub static FUNCTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(fn|def|function|func|sub|proc)\s+\w+").unwrap());

pub static TODO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(TODO|FIXME|HACK|XXX|BUG)\b").unwrap());

pub static GLOBAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(global\s|static\s+mut\s|@@\w+|\$[A-Z_]+)").unwrap());

pub static ASYNC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(async|await)\b").unwrap());

pub static SINGLETON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\.instance\(\)|getInstance|_instance\s*=|@instance|INSTANCE\s*=)").unwrap()
});

pub static SLEEP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(sleep|wait|timeout|delay)\s*\(").unwrap());

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

pub fn count_emojis(source: &str) -> usize {
    source.chars().filter(|&c| is_emoji(c)).count()
}

pub fn is_emoji(c: char) -> bool {
    let cp = c as u32;
    // Common emoji ranges (non-overlapping)
    matches!(
        cp,
        0x1F300..=0x1F9FF |  // Miscellaneous Symbols and Pictographs, Emoticons, Transport, etc.
        0x1F1E0..=0x1F1FF |  // Flags
        0x2600..=0x26FF |    // Miscellaneous Symbols
        0x2700..=0x27BF |    // Dingbats
        0x2300..=0x23FF |    // Miscellaneous Technical
        0x2B50..=0x2B55 |    // Stars
        0x203C..=0x3299      // Other symbols
    )
}

pub fn is_test_file(path: &str) -> bool {
    let p = path.to_lowercase();
    p.contains("test") || p.contains("spec") || p.ends_with("_test.go") || p.ends_with(".test.")
}

pub fn is_readme_file(path: &str) -> bool {
    let p = path.to_lowercase();
    p.contains("readme")
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
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
}
