# Rust Test Files for Dictator Linter

This directory contains intentionally malformed Rust files to test the Dictator linter's ability to detect structural violations.

## Files Overview

### 1. `01_trailing_whitespace.rs`
**Violation**: Trailing whitespace (spaces and tabs)
- Line 1: Comment with trailing spaces `violations   `
- Line 3: Field declaration with trailing spaces
- Line 9: Assignment with trailing spaces
- Line 14: println! statement with mixed trailing whitespace
- Line 24: Function with trailing spaces

**Code Status**: ✓ Syntactically valid Rust

### 2. `02_mixed_tabs_spaces.rs`
**Violation**: Mixed tabs and spaces for indentation
- Uses `\t` (tabs) for some indentation
- Uses spaces (4-space indentation) for others
- Mixed inconsistently throughout the file

**Code Status**: ✓ Syntactically valid Rust

Example:
```
\tenum Status {       // Tab indentation
    Inactive,         // Space indentation  
\tPending,           // Tab indentation again
}
```

### 3. `03_missing_final_newline.rs`
**Violation**: Missing final newline at end of file
- File ends with `}` (not `}\n`)
- Last 5 bytes: `)   )   ;  \n   }` (no newline after closing brace)

**Code Status**: ✓ Syntactically valid Rust

### 4. `04_visibility_order.rs`
**Violation**: Pub fields mixed with private fields (wrong visibility ordering)
- Struct `User` has pattern: `id (private), name (pub), email (private), age (pub)...`
- Struct `Permission` has: `role (private), can_edit (pub), read_only (private), can_delete (pub)`
- Methods also have mixed visibility: `fn private_init()` followed by `pub fn new()`

**Code Status**: ✓ Syntactically valid Rust

The anti-pattern is declaring `pub` fields after private ones, which violates grouping conventions.

### 5. `05_long_file.rs`
**Violation**: Long file (534 lines)
**Additional Violations**: 
- Contains mixed visibility ordering throughout
- Mixed public/private method declarations
- Demonstrates scale testing

**Code Status**: ✓ Syntactically valid Rust

## Testing with Dictator

These files are designed to test:

1. **Whitespace Detection**
   - Trailing spaces and tabs
   - Mixed indentation (tabs vs spaces)

2. **Line Ending Detection**
   - Missing final newline
   - Proper file termination

3. **Structural Analysis**
   - Visibility ordering (pub after private)
   - Field grouping conventions

4. **Scale Testing**
   - Long file handling
   - Performance with 500+ line files

## Running Tests

```bash
# Test individual files
dictator lint sandbox/rust/01_trailing_whitespace.rs
dictator lint sandbox/rust/02_mixed_tabs_spaces.rs
dictator lint sandbox/rust/03_missing_final_newline.rs
dictator lint sandbox/rust/04_visibility_order.rs
dictator lint sandbox/rust/05_long_file.rs

# Test all Rust files
dictator lint sandbox/rust/*.rs
```

## Expected Behavior

Dictator should report:
- Trailing whitespace violations (lines with trailing spaces/tabs)
- Inconsistent indentation (mixing tabs and spaces)
- Missing final newline
- Visibility ordering issues (pub after private in struct/impl)
- File size warnings (for files > some threshold)
