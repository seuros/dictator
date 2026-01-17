//! Visibility ordering checks (pub items before private).

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

use crate::counting::is_comment_only_line;

/// Rule 2: Visibility ordering - pub items should come before private items
pub fn check_visibility_ordering(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut line_start = 0;
    let mut in_struct = false;
    let mut in_impl = false;
    let mut has_private = false;
    let mut in_raw_string = false;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        // Track raw string literals (r" or r#")
        // Simple heuristic: line starting with `let ... = r"` or ending with `";` for multiline
        if !in_raw_string && (trimmed.contains("= r\"") || trimmed.contains("= r#\"")) {
            // Check if the raw string closes on the same line
            let after_open = trimmed.find("= r\"").map_or_else(
                || trimmed.find("= r#\"").map_or("", |pos| &trimmed[pos + 5..]),
                |pos| &trimmed[pos + 4..],
            );
            // If there's no closing quote on this line, we're in a multiline raw string
            if !after_open.contains('"') {
                in_raw_string = true;
            }
        } else if in_raw_string && (trimmed.ends_with("\";") || trimmed == "\";" || trimmed == "\"")
        {
            in_raw_string = false;
            line_start = nl + 1;
            continue;
        }

        // Skip lines inside raw string literals
        if in_raw_string {
            line_start = nl + 1;
            continue;
        }

        // Track struct/impl blocks
        if trimmed.contains("struct ") && trimmed.contains('{') {
            in_struct = true;
            has_private = false;
        } else if trimmed.contains("impl ") && trimmed.contains('{') {
            in_impl = true;
            has_private = false;
        } else if trimmed == "}" || trimmed.starts_with("}\n") {
            in_struct = false;
            in_impl = false;
            has_private = false;
        }

        // Check visibility within struct/impl
        if (in_struct || in_impl) && !trimmed.is_empty() && !is_comment_only_line(trimmed) {
            let is_pub = trimmed.starts_with("pub ");
            let is_field_or_method = is_struct_field_or_impl_item(trimmed);

            if is_field_or_method {
                if !is_pub && !has_private {
                    has_private = true;
                } else if is_pub && has_private {
                    diags.push(Diagnostic {
                        rule: "rust/visibility-order".to_string(),
                        message:
                            "Public item found after private item. Expected all public items first"
                                .to_string(),
                        enforced: false,
                        span: Span::new(line_start, nl),
                    });
                }
            }
        }

        line_start = nl + 1;
    }
}

/// Check if line is a struct field or impl method/associated function
fn is_struct_field_or_impl_item(trimmed: &str) -> bool {
    // Struct fields typically have pattern: [pub] name: Type [,]
    // Impl items typically have pattern: [pub] fn name(...) or [pub] const/type
    // Exclude closing braces, empty lines, attributes, and comments
    if trimmed.is_empty()
        || trimmed == "}"
        || trimmed.starts_with('}')
        || trimmed.starts_with('#')
        || trimmed.starts_with("//")
    {
        return false;
    }

    // Check for impl items (fn, const, type, unsafe, etc.)
    // These are more specific patterns, check them first
    if trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("pub const ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("pub unsafe fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("pub async fn ")
    {
        return true;
    }

    // Check for struct field pattern: identifier followed by colon and type
    // Must start with a valid Rust identifier (letter or underscore, optionally prefixed with pub)
    let field_part = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
    field_part.find(':').is_some_and(|colon_pos| {
        let before_colon = field_part[..colon_pos].trim();
        // Valid field name: starts with letter/underscore, contains only alphanumeric/underscore
        !before_colon.is_empty()
            && before_colon
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && before_colon
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

#[cfg(test)]
mod tests {
    use crate::lint_source;

    #[test]
    fn detects_pub_after_private_in_struct() {
        let src = r"
struct User {
    name: String,
    age: u32,
    pub email: String,
}
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should detect pub field after private fields in struct"
        );
    }

    #[test]
    fn detects_pub_after_private_in_impl() {
        let src = r"
impl User {
    fn private_method(&self) {}
    pub fn public_method(&self) {}
}
";
        let diags = lint_source(src);
        assert!(
            diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should detect pub method after private method in impl"
        );
    }

    #[test]
    fn accepts_pub_before_private() {
        let src = r"
struct User {
    pub id: u32,
    pub name: String,
    email: String,
}
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should accept public fields before private fields"
        );
    }

    #[test]
    fn accepts_impl_with_correct_order() {
        let src = r"
impl User {
    pub fn new(name: String) -> Self {
        User { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> bool {
        true
    }
}
";
        let diags = lint_source(src);
        assert!(
            !diags.iter().any(|d| d.rule == "rust/visibility-order"),
            "Should accept impl with public methods before private"
        );
    }
}
