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
mod tests;
