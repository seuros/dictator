# TypeScript Sandbox - Dictator Linter Test Files

This directory contains intentionally malformed TypeScript files for testing Dictator's linting and formatting capabilities.

## Files Created

### 1. BadReactComponent.tsx (56 lines)
A React component with multiple structural violations:
- Mixed tabs (lines 10, 15, 17, 19, 22, 24, 26, 27, 28, 31, 38, 44, 52) and spaces
- Trailing whitespace on multiple lines
- Inconsistent indentation within function blocks
- No final newline

### 2. UtilityFunctions.ts (151 lines)
A utility library with import ordering violations:
- Imports are randomly grouped instead of organized (lines 1-10):
  - Third-party: date-fns, lodash, uuid
  - System: fs, crypto, path
  - Third-party: axios
  - Local: types, config, validators
- Expected order: system → third-party → local
- No final newline

### 3. TooLongFile.ts (394 lines)
File that significantly exceeds recommended length:
- Contains 394 lines of code
- Multiple classes and interfaces that could be split
- Mixed line endings (CRLF and LF)
- Valid TypeScript syntax despite violations
- No final newline

### 4. InconsistentIndentation.ts (208 lines)
Severely inconsistent indentation throughout:
- Variable spacing indentation (1-8 spaces per level)
- Mixed tabs and spaces on multiple lines
- Inconsistent indentation within class methods
- Trailing whitespace on multiple lines
- No final newline

## Violation Summary

| File | Lines | Tabs/Spaces | Trailing WS | Import Order | File Length | Line Endings | Final NL |
|------|-------|-------------|-------------|--------------|-------------|------|----------|
| BadReactComponent.tsx | 56 | ✓ Mixed | ✓ Multiple | - | - | - | Missing |
| UtilityFunctions.ts | 151 | - | - | ✓ Disordered | - | - | Missing |
| TooLongFile.ts | 394 | - | - | - | ✓ Too Long | ✓ Mixed | Missing |
| InconsistentIndentation.ts | 208 | ✓ Mixed | ✓ Multiple | - | - | - | Missing |

**Total Violations: 40+**

## Testing Strategy

These files test Dictator's ability to detect:
1. Mixed tabs and spaces in indentation
2. Trailing whitespace at end of lines
3. Missing final newline in files
4. Files exceeding recommended length
5. Import statements not properly grouped
6. Mixed line ending formats (LF vs CRLF)
7. Inconsistent indentation levels

All files contain syntactically valid TypeScript to ensure violations are purely structural.

## Usage

```bash
# Run Dictator against these test files
dictator /sandbox/typescript

# Or specific file
dictator /sandbox/typescript/BadReactComponent.tsx

# Check with verbose output
dictator --verbose /sandbox/typescript
```

## Expected Output

Dictator should report:
- 10+ violations in BadReactComponent.tsx
- 8+ violations in UtilityFunctions.ts
- 6+ violations in TooLongFile.ts
- 15+ violations in InconsistentIndentation.ts

Total: 40+ structural violations
