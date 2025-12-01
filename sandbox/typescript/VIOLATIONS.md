# TypeScript Sandbox Violation Report

This directory contains intentionally malformed TypeScript files for testing Dictator linter violations.

## Files Overview

### 1. BadReactComponent.tsx

**Structural Violations:**
- **Mixed Tabs and Spaces**: Lines contain inconsistent indentation mixing tabs (\t) and spaces
  - Line 10: Tab before `name: string;`
  - Line 15: Tab indentation for state declarations
  - Line 19: Inconsistent spacing with tabs
  - Line 44: Tab indentation mixed with spaces
  - Line 52: Tab indentation mixed with spaces

- **Inconsistent Indentation**: Function bodies use different indentation styles
  - Some lines use 2-space indentation
  - Other lines use tab indentation
  - Conditional blocks have misaligned indentation

- **Trailing Whitespace**: Multiple lines end with trailing spaces
  - Line 7, 13, 18, 40, 55, 57 contain trailing spaces

- **No Final Newline**: File does not end with a final newline character

**Code Style:**
- Valid React component with TypeScript
- Proper imports but indentation is broken
- All TypeScript syntax is correct

---

### 2. UtilityFunctions.ts

**Structural Violations:**
- **Wrong Import Ordering**: Imports are randomly grouped instead of following convention
  - Line 1-2: date-fns, lodash (third-party)
  - Line 3-7: Mixed third-party and local imports
  - Line 8-10: System imports scattered throughout
  - Line 11: axios at end (should be grouped with other third-party)

  Expected order:
  1. System imports (fs, path, crypto)
  2. Third-party imports (axios, date-fns, lodash, uuid)
  3. Local imports (config, validators, types)

- **No Final Newline**: File ends at line 127 without final newline

- **Random Import Grouping**: No blank lines separating import categories

**Code Style:**
- All TypeScript syntax is valid
- Contains working utility functions
- Proper type annotations and interfaces

---

### 3. TooLongFile.ts

**Structural Violations:**
- **File Too Long**: 370+ lines exceeds typical file length limits
  - Contains complete implementations across multiple classes
  - Could be split into multiple focused modules
  - Reduces maintainability and readability

- **Inconsistent Line Endings**: Mixed line endings throughout
  - Some lines use Unix (LF) format
  - Some lines use Windows (CRLF) format
  - Creates git diff noise

- **No Final Newline**: File ends without final newline

**Code Style:**
- Valid TypeScript with Angular decorators
- Multiple repository classes (UserRepository, ProductRepository, OrderRepository)
- Service layer (ApiService)
- Utility classes (EventBus, StateManager, RequestQueue)
- Helper functions

---

### 4. InconsistentIndentation.ts

**Structural Violations:**
- **Severely Inconsistent Indentation**: Indentation varies wildly throughout the file
  - Lines 32-33: Minimal indentation (1 space)
  - Lines 39-41: Tab indentation
  - Lines 43-46: Excessive leading spaces (multiple indentation levels)
  - Line 49: Inconsistent spacing with tabs and spaces
  - Lines 56, 60, 70, 76-78: Random indentation shifts

- **Mixed Tabs and Spaces**: Throughout the file
  - Some lines use tabs
  - Other lines use spaces
  - Not consistent within logical blocks

- **No Final Newline**: File ends without final newline

- **Trailing Whitespace**: Lines 105, 118, 132, 145, 164, 176 contain trailing spaces

**Code Style:**
- Valid TypeScript with utility classes
- Configuration, validation, and utility helpers
- Proper type annotations despite formatting issues

---

## Testing Strategy

These files are designed to test Dictator's ability to detect:

1. **Tab vs Space Violations**: Consistent indentation style enforcement
2. **Trailing Whitespace**: Line ending cleanup
3. **Missing Final Newline**: File format standardization
4. **File Length**: Module organization
5. **Import Ordering**: Consistent import grouping
6. **Mixed Line Endings**: Platform-independent line ending handling
7. **Inconsistent Indentation**: Structural consistency

All files contain syntactically valid TypeScript to ensure violations are structural, not syntactic.

## Expected Dictator Output

When running Dictator on this directory, expected violations:

- BadReactComponent.tsx: 10+ violations
- UtilityFunctions.ts: 8+ violations
- TooLongFile.ts: 6+ violations
- InconsistentIndentation.ts: 15+ violations

Total: 40+ structural violations across 4 files
