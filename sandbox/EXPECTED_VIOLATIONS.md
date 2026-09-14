# Expected Violations in Sandbox

This document maps which decrees catch which violations when running
`dictator lint sandbox/`.

**This contract is enforced by tests.** Each language crate has a
`tests/sandbox.rs` integration test that lints these files and asserts the
violations below. If you edit a fixture (or your editor "helpfully" strips
trailing whitespace / normalizes line endings), the suite fails.
`.gitattributes` marks `sandbox/**` as binary-ish (`-text`) so git never
normalizes the bytes either.

Line limits count **code lines only** — blank lines and comments are excluded.

## decree.supreme (Universal)

### Trailing Whitespace
- `golang/trailing_whitespace.go` - Trailing spaces/tabs after code lines
- `python/trailing_whitespace.py` - 6 lines with trailing spaces
- `rust/01_trailing_whitespace.rs` - 7 lines with trailing whitespace
- `ruby/trailing_whitespace.rb` - Trailing spaces and a trailing tab
- `typescript/BadReactComponent.tsx` - 5 lines with trailing spaces

### Mixed Tabs/Spaces
- `golang/mixed_tabs_spaces.go` - Spaces in a Go file (should be tabs)
- `ruby/mixed_indentation.rb` - Tabs mixed with spaces
- `python/mixed_indentation.py` - Mixed tabs and spaces
- `typescript/BadReactComponent.tsx` - 18 tab characters mixed with spaces
- `typescript/InconsistentIndentation.ts` - Severely inconsistent (1-8 spaces)
- `rust/02_mixed_tabs_spaces.rs` - Tab and space indentation mixed

### Missing Final Newline
- `golang/missing_newline.go` - File ends with `}`, no newline
- `python/no_final_newline.py` - Missing final newline
- `rust/03_missing_final_newline.rs` - File ends with `}`, no newline
- `ruby/no_final_newline.rb` - No newline at EOF
- `typescript/TooLongFile.ts` - Missing final newline
- `typescript/BadReactComponent.tsx` - Missing final newline
- `typescript/UtilityFunctions.ts` - Missing final newline
- `typescript/InconsistentIndentation.ts` - Missing final newline

### Mixed Line Endings
- `golang/mixed_line_endings.go` - Alternating CRLF/LF
- `python/long_file_mixed_endings.py` - Mixed CRLF/LF
- `ruby/mixed_line_endings.rb` - Alternating CRLF/LF
- `typescript/TooLongFile.ts` - Mixed CRLF and LF

## decree.ruby

Ruby wraps supreme rules under the `ruby/` prefix, plus:

### File Too Long (max 300 code lines)
- `ruby/too_long_file.rb` - 316 code lines

### Comment Spacing
- `ruby/wrong_comment_spacing.rb` - 13 instances of `#comment` instead of
  `# comment` (shebang and magic comments exempt)

## decree.typescript

### File Too Long (max 350 code lines)
- `typescript/TooLongFile.ts` - Exceeds the limit

### Import Ordering
- `typescript/UtilityFunctions.ts` - Import lines randomly grouped
- `typescript/WrongImportOrder.ts` - Wrong import grouping

### Inconsistent Indentation
- `typescript/InconsistentIndentation.ts` - 20+ instances of varying indentation

## decree.golang

### File Too Long (max 450 code lines)
- `golang/long_file.go` - 373 lines (under limit, should pass)

### Tabs Required (not spaces)
- `golang/mixed_tabs_spaces.go` - Lines using spaces instead of tabs

### Raw String Exemption (should pass)
- `golang/raw_string_help.go` - Space-indented help text inside backtick
  strings must NOT be flagged

## decree.rust

### File Too Long (max 400 code lines)
- `rust/05_long_file.rs` - 534 lines

### Visibility Ordering
- `rust/04_visibility_order.rs` - Public after private in structs and methods
- `rust/05_long_file.rs` - Multiple visibility ordering violations

### Fossil Edition (opt-in via `min_edition`)
- `rust/old_edition_cargo.toml` - Declares edition 2021; flagged when
  `min_edition = "2024"` is configured

## decree.python

### File Too Long (max 380 code lines, blanks/comments excluded)
- `python/long_file_mixed_endings.py` - 450 physical lines but only 337 code
  lines (under limit, should pass)

### Import Ordering (PEP 8)
- `python/wrong_import_order.py` - Third-party before stdlib

## decree.frontmatter

**Applies to:** `.md`, `.mdx` files only (YAML frontmatter)

Tests use the blog contract: order `title, slug, pubDate, description, tags`,
required `title, slug`.

### Frontmatter Field Order
- `configs/blog-wrong-frontmatter-order.md` - pubDate before title
- `configs/blog-multiple-violations.md` - pubDate and description before title
- `configs/component-wrong-order.mdx` - tags and description before title

### Missing Required Fields
- `configs/blog-missing-required-field.md` - Missing `slug` field
- `configs/blog-multiple-violations.md` - Missing `slug` field

### Invalid YAML Frontmatter
- `configs/blog-invalid-yaml.md` - Broken YAML syntax (unclosed array)

### Valid Files (No Violations)
- `configs/blog-valid-frontmatter.md` - Correctly ordered frontmatter
- `configs/blog-no-frontmatter.md` - No frontmatter to validate (ignored)

## Not Implemented (fixtures kept for future decrees)

- `golang/wrong_package.go` - Package/structure validation: no
  `golang/wrong-package` rule exists yet. The file still exercises supreme
  rules like any other Go file.
- Ruby method visibility ordering: no rule exists yet (rust has
  `rust/visibility-order`; ruby does not).
