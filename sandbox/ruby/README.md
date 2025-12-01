# Dictator Linter Test Files - Structural Violations

This directory contains intentionally malformed Ruby files for testing Dictator's linting capabilities. All files are syntactically valid Ruby code but contain structural violations.

## Test Files

### 1. `trailing_whitespace.rb`
**Violation**: Trailing whitespace on lines
- Lines 1 and 4 contain trailing spaces
- Should be detected by rules checking for trailing whitespace

### 2. `mixed_indentation.rb`
**Violation**: Mixed tabs and spaces for indentation
- Lines 2 and 6 use TAB characters
- Lines 3, 5, and others use spaces
- Inconsistent indentation style throughout

### 3. `no_final_newline.rb`
**Violation**: Missing final newline at end of file
- File ends without a newline character
- Should trigger rule checking for final newline requirement

### 4. `too_long_file.rb`
**Violation**: Excessive file length (316 lines)
- ProductsController class with 40+ utility methods
- Exceeds typical file length guidelines (often 200-250 lines)
- Could trigger complexity/length rules

### 5. `wrong_comment_spacing.rb`
**Violation**: Incorrect comment spacing
- Comments use `#comment` instead of `# comment`
- 13 violations throughout the file (lines 2, 4, 8, 10, 16, 18, 24, 26, 37, 42)
- Should be caught by comment formatting rules

### 6. `visibility_order_wrong.rb`
**Violation**: Wrong method visibility ordering
- `private` methods defined BEFORE `public` methods (lines 4-16)
- `public` keyword appears after private methods (line 20)
- `protected` methods appear at the end (line 57)
- Proper order should be: public → protected → private

### 7. `mixed_line_endings.rb`
**Violation**: Mixed line endings (CRLF and LF)
- Even-numbered lines use CRLF (Windows-style)
- Odd-numbered lines use LF (Unix-style)
- Should trigger line-ending consistency checks

## Syntax Validity

All files are syntactically valid Ruby and will parse without errors. The violations are purely structural and style-related, making them ideal test cases for a linter focused on code structure and formatting standards rather than syntax correctness.

## Usage

Run Dictator against these files to validate detection of:
- Whitespace issues
- Indentation consistency
- File structure and length
- Comment formatting
- Method visibility ordering
- Line ending consistency
