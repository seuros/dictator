# Go Linter Test Files - Structural Violations

This directory contains intentionally malformed Go files for testing the Dictator linter. All files are syntactically valid Go but have structural violations.

## Files and Violations

### 1. `trailing_whitespace.go`
- **Violation**: Trailing whitespace at end of lines
- **Details**: Lines contain trailing spaces after code/comments
- **Line count**: 13 lines
- **Expected detection**: Linter should flag trailing whitespace

### 2. `mixed_tabs_spaces.go`
- **Violation**: Mixed tabs and spaces for indentation
- **Details**: Go convention mandates tabs for indentation only. This file mixes spaces with tabs:
  - Line 8: Uses spaces before tab for indentation
  - Line 14: Uses spaces for indentation instead of tab
  - Line 20: Uses spaces instead of tab
- **Line count**: 23 lines
- **Expected detection**: Linter should flag mixed indentation

### 3. `missing_newline.go`
- **Violation**: Missing final newline at end of file
- **Details**: File ends with `}` without a trailing newline character
- **Line count**: 17 lines (not counting missing final newline)
- **Expected detection**: Linter should flag missing final newline

### 4. `long_file.go`
- **Violation**: Excessive file length
- **Details**: File is 373 lines long, exceeding typical single-responsibility principle
- **Additional violations**:
  - Mixed indentation in `Add()` method (line ~165)
  - Syntactically valid but structurally too large for a single file
- **Line count**: 373 lines
- **Expected detection**: Linter should flag files exceeding line limits (if configured)

### 5. `mixed_line_endings.go`
- **Violation**: Mixed line endings (CRLF and LF)
- **Details**: File contains both DOS (CRLF) and Unix (LF) line endings
- **File type**: Shows as "with CRLF, LF line terminators"
- **Line count**: 18 lines
- **Expected detection**: Linter should flag inconsistent line endings

### 6. `wrong_package.go`
- **Violation**: Wrong package name structure
- **Details**:
  - Package is declared as `package utils` (not matching filename context)
  - File is in `sandbox/golang/` but declares `package utils`
  - Contains a `main()` function which is invalid in a non-main package
- **Line count**: 48 lines
- **Expected detection**: Linter should flag package structure violations

## Testing Notes

All files are syntactically valid Go code that will compile (mostly), but they violate style/structure rules that a code formatter or linter should catch.

- **Compilation**: Files may compile with `go build` but will fail with proper `gofmt` or `golangci-lint`
- **Formatting**: `gofmt` will correct most issues; this is what Dictator should detect
- **Purpose**: Test that Dictator properly validates Go structural conventions
