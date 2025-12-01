# Dictator Rust Linter Test Suite

Complete sandbox of intentionally malformed Rust files for testing Dictator structural violations.

## Quick Start

```bash
# Test all Rust files
dictator lint sandbox/rust/

# Test individual files
dictator lint sandbox/rust/01_trailing_whitespace.rs
dictator lint sandbox/rust/02_mixed_tabs_spaces.rs
dictator lint sandbox/rust/03_missing_final_newline.rs
dictator lint sandbox/rust/04_visibility_order.rs
dictator lint sandbox/rust/05_long_file.rs

# Verify violations are present
dictator lint .
```

## Files

| File | Violations | Lines | Size |
|------|-----------|-------|------|
| 01_trailing_whitespace.rs | 7 trailing whitespace violations | 24 | 504B |
| 02_mixed_tabs_spaces.rs | Mixed tabs (16) and spaces (2) | 31 | 484B |
| 03_missing_final_newline.rs | Missing final newline | 39 | 857B |
| 04_visibility_order.rs | Pub after private (mixed visibility) | 99 | 2.0K |
| 05_long_file.rs | 534 lines, visibility ordering | 534 | 13K |

## Violation Details

### 01_trailing_whitespace.rs
**Type**: Trailing whitespace (spaces and tabs at end of lines)

Affected lines:
```
Line 1:  // Rust file with trailing whitespace violations   
Line 3:      pub name: String,   
Line 4:      pub version: String,	
Line 9:          Config { name, version }   
Line 14:         println!("Version: {}", self.version);  	
Line 20:         "MyApp".to_string(),   
Line 24: }   
```

**Validation**: `grep -n '[ \t]$' 01_trailing_whitespace.rs` returns 7 matches

---

### 02_mixed_tabs_spaces.rs
**Type**: Mixed indentation (tabs and spaces)

Pattern:
- Lines 3, 5, 7, 11, 13, etc.: Start with `\t` (tab)
- Lines 4, 6, 8, 12, 14, etc.: Start with spaces (4-space indent)

Example:
```rust
pub enum Status {
    \tActive,
        Inactive,  // 4 spaces instead of tab
    \tPending,
}
```

**Validation**: 16 lines with tab prefix, 2 lines with space prefix mixed throughout

---

### 03_missing_final_newline.rs
**Type**: File missing final newline

File ends with:
```
    dispatcher.dispatch("test".to_string());
}
```

Last byte is `}` (0x7D) instead of `}\n` (0x7D 0x0A)

**Validation**: `[ -z "$(tail -c 1 file.rs)" ]` returns true (no newline)

---

### 04_visibility_order.rs
**Type**: Wrong visibility ordering (pub after private)

Struct `User` (lines 2-10):
```rust
pub struct User {
    id: u32,                    // private
    pub name: String,           // public
    email: String,              // private
    pub age: u8,                // public
    created_at: String,         // private
    pub updated_at: String,     // public
    internal_state: bool,       // private
    pub metadata: String,       // public
}
```

**Pattern**: Private and public fields are alternating/mixed instead of grouped

**Violations**:
- User struct fields (8 violations)
- Permission struct fields (4 violations)
- Method visibility ordering throughout impl blocks

---

### 05_long_file.rs
**Type**: Large file (500+ lines) with multiple violations

**Size**: 534 lines, 13KB

**Contained Violations**:
- Visibility ordering (pub/private mixed throughout)
- Module organization with poor encapsulation
- Multiple impl blocks with visibility ordering issues

**Modules**:
- `models`: Product, Order, Warehouse structs
- `services`: ProductService, OrderService, WarehouseService impls
- `handlers`: EventHandler trait impls
- `advanced_services`: Analytics, Reporting, Validation, Cache
- `notification_service`: Email, SMS, Slack notifiers
- `database_models`: DB connection logic
- `logging`: Logger utility
- `testing`: Test runner utility

---

## Test Coverage

### Structural Violations
- [x] Trailing whitespace
- [x] Mixed indentation (tabs vs spaces)
- [x] Missing final newline
- [x] Visibility ordering (pub after private)
- [x] Large files (500+ lines)

### Code Validity
- [x] All files have syntactically valid Rust
- [x] No compiler errors (only structural issues)
- [x] All types and functions properly defined
- [x] Trait implementations valid

### Expected Dictator Output

Running `dictator lint sandbox/rust/` should report:

```
01_trailing_whitespace.rs:
  ✗ Trailing whitespace on line 1
  ✗ Trailing whitespace on line 3
  ✗ Trailing whitespace on line 4
  ✗ Trailing whitespace on line 9
  ✗ Trailing whitespace on line 14
  ✗ Trailing whitespace on line 20
  ✗ Trailing whitespace on line 24

02_mixed_tabs_spaces.rs:
  ✗ Inconsistent indentation on line 3 (tab)
  ✗ Inconsistent indentation on line 4 (spaces)
  [... more lines ...]

03_missing_final_newline.rs:
  ✗ Missing final newline

04_visibility_order.rs:
  ⚠ Public field after private field (line 3, 5, 7, 9)
  ⚠ Public method after private method (multiple locations)

05_long_file.rs:
  ⚠ File exceeds recommended line count: 534 lines
  ⚠ Public field/method after private (multiple violations)
```

## Verification

Run the verification script to confirm all violations are present:

```bash
dictator lint .
```

Output:
```
=== RUST VIOLATIONS VERIFICATION ===
Checking files in: sandbox/rust

1. Checking 01_trailing_whitespace.rs...
   Found 7 lines with trailing whitespace
   ✓ PASS

2. Checking 02_mixed_tabs_spaces.rs...
   Found 16 lines starting with tabs
   Found 2 lines starting with 4 spaces
   ✓ PASS

3. Checking 03_missing_final_newline.rs...
   File MISSING final newline
   ✓ PASS

4. Checking 04_visibility_order.rs...
   Found mixed pub/private field declarations
   ✓ PASS

5. Checking 05_long_file.rs...
   File has 534 lines
   ✓ PASS

=== VERIFICATION COMPLETE ===
```

## File Organization

```
sandbox/rust/
├── 01_trailing_whitespace.rs    (Whitespace violations)
├── 02_mixed_tabs_spaces.rs      (Indentation violations)
├── 03_missing_final_newline.rs  (EOF violation)
├── 04_visibility_order.rs       (Structural ordering)
├── 05_long_file.rs              (Scale + violations)
├── README.md                    (Detailed explanation)
├── INDEX.md                     (This file)
└── dictator lint    (Verification script)
```

## Notes

- All files are valid Rust code (compiles without syntax errors)
- Violations are intentional and documented
- Files are UTF-8 encoded with Unix (LF) line endings
- Suitable for automated testing of Dictator linter
- Can be extended with additional test cases as needed
