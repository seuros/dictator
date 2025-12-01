# Dictator TypeScript Sandbox - Testing Checklist

Use this checklist when running Dictator against the sandbox files to verify all violations are detected.

## File 1: BadReactComponent.tsx

Expected Violations:

- [ ] **Tab/Space Mixing**
  - [ ] Line 10: `	name: string;` (tab instead of spaces)
  - [ ] Line 15: `	const [loading, ...` (tab vs space inconsistency)
  - [ ] Line 17: `	const [error, ...` (tab vs space inconsistency)
  - [ ] Line 19: `	useEffect(() => {` (tab vs space inconsistency)
  - [ ] Lines 22, 24, 26, 27, 28, 31: Multiple tabs in nested blocks
  - [ ] Lines 38, 44, 52: Additional tab instances

- [ ] **Missing Final Newline**
  - [ ] File ends with `export default BadReactComponent;` without trailing newline

- [ ] **Inconsistent Indentation Depth**
  - [ ] Function body indentation varies throughout

Expected Count: 10+ violations

---

## File 2: UtilityFunctions.ts

Expected Violations:

- [ ] **Import Ordering Violations**
  - [ ] Line 1: Third-party `date-fns` (should follow system imports)
  - [ ] Line 3: `uuid` grouped with wrong category
  - [ ] Line 4-6: System imports (`fs`, `./types`, `./config`) scattered
  - [ ] Line 7-8: More system imports (`crypto`, `path`)
  - [ ] Line 10: `axios` separated from other third-party imports

Correct order should be:
  1. System imports: `crypto`, `fs`, `path`
  2. Third-party imports: `axios`, `date-fns`, `lodash`, `uuid`
  3. Local imports: `./config`, `./types`, `./validators`

- [ ] **Missing Final Newline**
  - [ ] File ends with `}` on line 151 without trailing newline

Expected Count: 8+ violations

---

## File 3: TooLongFile.ts

Expected Violations:

- [ ] **File Too Long**
  - [ ] File has 394 lines (exceeds ~250-350 line recommended maximum)
  - [ ] Contains multiple classes that could be split:
    - UserRepository (lines 51-88)
    - ProductRepository (lines 89-126)
    - OrderRepository (lines 127-164)
    - ApiService (lines 165-236)
    - EventBus (lines 237-273)
    - StateManager (lines 274-306)
    - RequestQueue (lines 307-343)

- [ ] **Mixed Line Endings**
  - [ ] File contains both CRLF and LF line endings
  - [ ] Detected via `file` command shows "with CRLF, LF line terminators"

- [ ] **Missing Final Newline**
  - [ ] File ends with closing `};` without trailing newline

Expected Count: 6+ violations

---

## File 4: InconsistentIndentation.ts

Expected Violations:

- [ ] **Severely Inconsistent Indentation**
  - [ ] Line 33: 1 space indentation (` return`)
  - [ ] Line 39: Tab indentation (`→	static`)
  - [ ] Line 47: 7 space indentation (`       return`)
  - [ ] Line 49: Mixed tab and spaces (` 	   return`)
  - [ ] Line 56: 1 space indent (` return`)
  - [ ] Line 70: Minimal indentation in method
  - [ ] Line 76-78: Variable indentation in camelCase method
  - [ ] Line 105: No indent (`  return`)
  - [ ] Line 118: Excessive indent for loop
  - [ ] Line 132: Inconsistent padding
  - [ ] Line 145: Over-indented return
  - [ ] Line 164: Inconsistent object literal indent
  - [ ] Line 176: Final method misaligned

- [ ] **Tab/Space Mixing**
  - [ ] Line 39: `→	static isBoolean` (tab after space)
  - [ ] Line 49: Mixed spaces and tabs
  - [ ] Line 105: Tab in whitespace

- [ ] **Missing Final Newline**
  - [ ] File ends with `};` without trailing newline

Expected Count: 15+ violations

---

## Aggregate Statistics

Total Files:        4 TypeScript files
Total Lines:        809 lines of code
Total Violations:   40+ structural violations

### Violation Breakdown

| Category | Count | Files |
|----------|-------|-------|
| Indentation Issues | 20+ | All 4 files |
| Tab/Space Mixing | 21 | BadReactComponent.tsx, InconsistentIndentation.ts |
| Import Ordering | 10 | UtilityFunctions.ts |
| File Length | 1 | TooLongFile.ts |
| Line Ending | Mixed | TooLongFile.ts |
| Missing Final NL | 4 | All 4 files |

---

## Dictator CLI Commands to Test

```bash
# Run against entire directory
dictator sandbox/typescript/

# Run against specific file
dictator sandbox/typescript/BadReactComponent.tsx

# Show violations in verbose mode
dictator --verbose sandbox/typescript/

# Auto-fix violations (if supported)
dictator --fix sandbox/typescript/

# Generate report
dictator --report json sandbox/typescript/ > report.json
```

---

## Expected Dictator Output

Sample expected format for violations detected:

```
BadReactComponent.tsx:10:1 - Mixed tabs and spaces detected
BadReactComponent.tsx:15:1 - Tab character in indentation
BadReactComponent.tsx:17:1 - Inconsistent indentation
BadReactComponent.tsx:56:1 - Missing final newline
UtilityFunctions.ts:1:1 - Import not in correct order (third-party before system)
UtilityFunctions.ts:4:1 - Import not in correct order (local mixed with system)
UtilityFunctions.ts:151:1 - Missing final newline
TooLongFile.ts:1:1 - File exceeds 350 lines (current: 394)
TooLongFile.ts:1:1 - Mixed line endings detected (CRLF and LF)
TooLongFile.ts:394:1 - Missing final newline
InconsistentIndentation.ts:33:1 - Inconsistent indentation (1 space vs standard)
InconsistentIndentation.ts:39:1 - Mixed tabs and spaces
... (40+ total violations)
```

---

## Verification Steps

1. Run Dictator on each file individually
2. Verify all expected violations are reported
3. Check line numbers match documentation
4. Verify severity levels are appropriate
5. Test --fix functionality if available
6. Verify files can be auto-corrected

---

## Notes for Dictator Developers

- All TypeScript syntax is valid; violations are purely structural
- Files use realistic code (React components, utilities, services)
- Violations are intentional and documented for reproducibility
- These files should work with any TypeScript linter configuration
- Consider testing with various `.dictate.toml` configurations
- Verify linter behavior with different indent widths (2, 4 spaces)
