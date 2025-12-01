//! Enthusiasm and joy enforcement rules
//!
//! The Party demands visible displays of revolutionary fervor.

use crate::config::KjrConfig;
use crate::helpers::count_emojis;
use dictator_decree_abi::{Diagnostic, Diagnostics, Span};

/// kjr/insufficient-joy - Files must contain mandatory enthusiasm markers
pub fn check_insufficient_joy(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    let emoji_count = count_emojis(source);
    if emoji_count < config.min_emojis {
        let msg = format!(
            "File contains {} emoji(s). The Party requires at least {}. \
             Excessive seriousness is counter-revolutionary.",
            emoji_count, config.min_emojis
        );
        diags.push(Diagnostic {
            rule: "kjr/insufficient-joy".into(),
            message: msg,
            enforced: true,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

/// kjr/missing-dear-leader-comment - Files must acknowledge leadership
pub fn check_missing_dear_leader(source: &str, config: &KjrConfig, diags: &mut Diagnostics) {
    let first_lines: String = source
        .lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();

    let has_praise = config
        .praise_keywords
        .iter()
        .any(|kw| first_lines.contains(&kw.to_lowercase()));

    if !has_praise {
        let msg = "Every file must open with a loyal hymn to the Dictator. \
                   Add praise keywords: Kim, Supreme, Glorious, etc.";
        diags.push(Diagnostic {
            rule: "kjr/missing-dear-leader-comment".into(),
            message: msg.into(),
            enforced: false,
            span: Span::new(0, source.len().min(50)),
        });
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
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
}
