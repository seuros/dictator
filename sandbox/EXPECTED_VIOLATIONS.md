# Expected Violations in Sandbox

This document maps which decrees should catch which violations when running `dictator lint sandbox/`.

## decree.supreme (Universal)

Should catch violations in **ALL** files:

### Trailing Whitespace
- `ruby/trailing_whitespace.rb` - Lines with trailing spaces
- `typescript/BadReactComponent.tsx` - Trailing whitespace
- `golang/trailing_whitespace.go` - Trailing spaces/tabs
- `rust/01_trailing_whitespace.rs` - 7 lines with trailing whitespace
- `python/trailing_whitespace.py` - 6 lines with trailing spaces
- `configs/blog-trailing-whitespace.md` - Trailing spaces in frontmatter

### Mixed Tabs/Spaces
- `ruby/mixed_indentation.rb` - Tabs + spaces mixed
- `typescript/BadReactComponent.tsx` - 18 tab characters mixed with spaces
- `typescript/InconsistentIndentation.ts` - Severely inconsistent (1-8 spaces)
- `golang/mixed_tabs_spaces.go` - Spaces in Go file (should be tabs)
- `rust/02_mixed_tabs_spaces.rs` - 16 lines tabs, 2 lines spaces
- `python/mixed_indentation.py` - Mixed tabs and spaces
- `configs/docker-compose-mixed-indentation.yml` - TAB on line 7
- `configs/config-tabs-and-spaces.toml` - Mixed indentation

### Missing Final Newline
- `ruby/no_final_newline.rb` - No newline at EOF
- `typescript/BadReactComponent.tsx` - Missing final newline
- `typescript/UtilityFunctions.ts` - Missing final newline
- `typescript/TooLongFile.ts` - Missing final newline
- `typescript/InconsistentIndentation.ts` - Missing final newline
- `golang/missing_newline.go` - No final newline
- `rust/03_missing_final_newline.rs` - File ends with `}`, no newline
- `python/no_final_newline.py` - Missing final newline
- `configs/blog-missing-final-newline.md` - No EOF newline

### Mixed Line Endings
- `ruby/mixed_line_endings.rb` - Alternating CRLF/LF
- `typescript/TooLongFile.ts` - Mixed CRLF and LF
- `golang/mixed_line_endings.go` - CRLF + LF mixed
- `python/long_file_mixed_endings.py` - Mixed CRLF/LF
- `configs/app-config-mixed-line-endings.toml` - Mixed line endings

### Blank Line Whitespace
- Files with spaces/tabs on blank lines (multiple files)

## decree.ruby

### File Too Long (max 300 lines)
- `ruby/too_long_file.rb` - 316 lines (ProductsController)

### Comment Spacing
- `ruby/wrong_comment_spacing.rb` - 13 instances of `#comment` instead of `# comment`

### Method Visibility Ordering
- `ruby/visibility_order_wrong.rb` - Private before public, protected at end

## decree.typescript

### File Too Long (max 350 lines)
- `typescript/TooLongFile.ts` - 394 lines

### Import Ordering
- `typescript/UtilityFunctions.ts` - 10 import lines randomly grouped

### Inconsistent Indentation
- `typescript/InconsistentIndentation.ts` - 20+ instances of varying indentation

## decree.golang

### File Too Long (max 450 lines)
- `golang/long_file.go` - 373 lines (under limit, should pass)

### Tabs Required (not spaces)
- `golang/mixed_tabs_spaces.go` - Lines 8, 14, 20 use spaces instead of tabs

### Wrong Package Structure
- `golang/wrong_package.go` - Package declared as `utils` but contains `main()`

## decree.rust

### File Too Long (max 400 lines)
- `rust/05_long_file.rs` - 534 lines

### Visibility Ordering
- `rust/04_visibility_order.rs` - Public after private in structs and methods
- `rust/05_long_file.rs` - Multiple visibility ordering violations

## decree.python

### File Too Long (max 380 lines)
- `python/long_file_mixed_endings.py` - 450 lines

### Import Ordering (PEP 8)
- `python/wrong_import_order.py` - Third-party before stdlib

## decree.frontmatter

**Applies to:** `.md`, `.mdx` files only (YAML frontmatter)
**Does NOT apply to:** `.astro`, `.yml`, `.yaml`, `.toml` (need separate decrees)

### Frontmatter Field Order
- `configs/blog-wrong-frontmatter-order.md` - pubDate before title (expected: title, slug, pubDate, description, tags)
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


## Summary Statistics

**Total Expected Violations: 75+**

By decree:
- `decree.supreme`: ~45 violations (trailing whitespace, tabs/spaces, final newline, line endings)
- `decree.ruby`: ~5 violations (file length, comment spacing, visibility order)
- `decree.typescript`: ~8 violations (file length, import order, indentation)
- `decree.golang`: ~4 violations (tabs vs spaces, package structure)
- `decree.rust`: ~6 violations (file length, visibility order)
- `decree.python`: ~3 violations (file length, import order)
- `decree.frontmatter`: ~8 violations (field order, missing fields, invalid YAML)

When Dictator is fully implemented, running:
```bash
dictator lint sandbox/
```

Should output exactly these violations with proper file paths, line numbers, and severity levels.
