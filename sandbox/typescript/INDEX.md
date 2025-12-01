# TypeScript Test Sandbox - File Index

Complete reference for the Dictator TypeScript linting sandbox.

## Quick Navigation

### Test Files (4)
1. **BadReactComponent.tsx** - React component with indentation violations
2. **UtilityFunctions.ts** - Utilities with import ordering issues
3. **TooLongFile.ts** - Oversized file with mixed line endings
4. **InconsistentIndentation.ts** - Severe indentation inconsistencies

### Documentation (4)
1. **README.md** - Overview and quick start guide
2. **VIOLATIONS.md** - Detailed violation catalog
3. **VIOLATION_EXAMPLES.md** - Code examples with line numbers
4. **TESTING_CHECKLIST.md** - Verification checklist
5. **INDEX.md** - This file

---

## File Details

### 1. BadReactComponent.tsx
**Purpose:** Test React component formatting violations

**Violations:**
- 18 tab characters in indentation
- Inconsistent indentation within function bodies
- Missing final newline
- Total: 10+ violations

**Key Issues:**
- Line 10: Tab character (`\t`) instead of spaces
- Line 15-17: Mixed tabs and spaces in variable declarations
- Line 22-31: Nested block indentation with tabs
- Line 44, 52: JSX indentation violations

**Code Type:** React component with TypeScript
**Syntax:** Valid
**Size:** 56 lines, 1.3 KB

---

### 2. UtilityFunctions.ts
**Purpose:** Test import statement ordering violations

**Violations:**
- 10 import lines in wrong order
- System/third-party/local imports mixed
- Missing final newline
- Total: 8+ violations

**Key Issues:**
- Line 1-3: Third-party imports before system imports
- Line 4-6: System imports mixed with local imports
- Line 7-8: Additional system imports scattered
- Line 10: Third-party axios separated from others

**Code Type:** Utility library with functions
**Syntax:** Valid
**Size:** 151 lines, 3.6 KB

**Correct Import Order:**
```typescript
// 1. System imports
import * as fs from 'fs'
import * as path from 'path'
import crypto from 'crypto'

// 2. Third-party imports
import axios from 'axios'
import { format } from 'date-fns'
import lodash from 'lodash'
import { v4 as uuidv4 } from 'uuid'

// 3. Local imports
import type { Logger } from './types'
import { config } from './config'
import { validateInput } from './validators'
```

---

### 3. TooLongFile.ts
**Purpose:** Test file length and line ending violations

**Violations:**
- 394 lines (exceeds 350+ limit)
- Mixed CRLF and LF line endings
- Multiple classes that should be split
- Missing final newline
- Total: 6+ violations

**Key Issues:**
- Contains 7 major classes/components
- Could be split into 3-4 separate files:
  - repositories.ts (UserRepository, ProductRepository, OrderRepository)
  - services.ts (ApiService)
  - events.ts (EventBus, StateManager)
  - queue.ts (RequestQueue)
  - utilities.ts (Helper functions)

**Code Type:** Service layer with repositories
**Syntax:** Valid (Angular-compatible)
**Size:** 394 lines, 9.6 KB

---

### 4. InconsistentIndentation.ts
**Purpose:** Test severe indentation inconsistency

**Violations:**
- 3 tab characters in indentation
- Indentation depth varying from 1-8 spaces per level
- Inconsistent spacing within class methods
- Missing final newline
- Total: 15+ violations

**Key Issues:**
- Line 33: Only 1 space indentation (` return`)
- Line 39: Tab indentation (`\tstatic`)
- Line 47: 7 spaces indentation (`       return`)
- Line 49: Mixed tab and spaces (` \t   return`)
- Multiple methods with different indent levels

**Code Type:** Utility classes (Configuration, Validators, StringUtils, etc.)
**Syntax:** Valid
**Size:** 208 lines, 5.2 KB

---

## Documentation Guide

### README.md
- Project overview
- File summaries
- Violation table
- Testing strategy
- Usage examples

### VIOLATIONS.md
- Organized by file
- Line number references
- Specific violation types
- Expected violation counts

### VIOLATION_EXAMPLES.md
- Exact code snippets
- Line-by-line analysis
- Visual violation indicators
- Comparison with correct code
- Summary tables

### TESTING_CHECKLIST.md
- Checkbox verification items
- Expected CLI commands
- Output format examples
- Verification steps
- Notes for developers

### INDEX.md
- This file
- Quick navigation
- Detailed file summaries
- Complete reference

---

## Violation Statistics

```
Total Files:              4
Total Lines of Code:      809
Total Violations:         40+

By Category:
  Tab/Space Mixing:       21 instances
  Inconsistent Indent:    20+ instances
  Import Ordering:        10 lines
  File Too Long:          1 file
  Line Endings Mixed:     1 file
  Missing Final NL:       4 files

By File:
  BadReactComponent.tsx:          10+ violations
  UtilityFunctions.ts:            8+ violations
  TooLongFile.ts:                 6+ violations
  InconsistentIndentation.ts:     15+ violations
```

---

## Testing Workflow

### Step 1: Run Dictator
```bash
dictator sandbox/typescript/
```

### Step 2: Check Individual Files
```bash
dictator sandbox/typescript/BadReactComponent.tsx
dictator sandbox/typescript/UtilityFunctions.ts
dictator sandbox/typescript/TooLongFile.ts
dictator sandbox/typescript/InconsistentIndentation.ts
```

### Step 3: Verify Violations
Use TESTING_CHECKLIST.md to verify all expected violations are detected.

### Step 4: Test Fixes
If Dictator supports auto-fix:
```bash
dictator --fix sandbox/typescript/
```

### Step 5: Validate Results
Use VIOLATION_EXAMPLES.md to validate fix accuracy.

---

## Test Coverage

These files test Dictator's ability to detect and report:

- Indentation consistency (tabs vs spaces)
- Indentation depth consistency
- Trailing whitespace handling
- Final newline requirements
- Line ending normalization
- Import statement ordering
- Import category grouping
- File length enforcement
- Module organization
- React component formatting
- TypeScript type annotations
- Generic type parameters

---

## Code Quality Notes

All test files:
- ✓ Have valid TypeScript syntax
- ✓ Contain realistic, working code
- ✓ Include proper type annotations
- ✓ Follow TypeScript conventions
- ✓ Have realistic imports
- ✓ Contain actual implementations
- ✓ Use modern patterns (hooks, async/await, generics)

Violations are purely structural, not syntactic.

---

## File Structure Summary

```
sandbox/typescript/
├── BadReactComponent.tsx              (React component - 56 lines)
├── UtilityFunctions.ts                (Utility library - 151 lines)
├── TooLongFile.ts                     (Service layer - 394 lines)
├── InconsistentIndentation.ts         (Utility classes - 208 lines)
├── README.md                          (Quick start)
├── VIOLATIONS.md                      (Violation catalog)
├── VIOLATION_EXAMPLES.md              (Code examples)
├── TESTING_CHECKLIST.md               (Verification checklist)
└── INDEX.md                           (This file)
```

---

## Support References

- React/TypeScript patterns in BadReactComponent.tsx
- Import grouping conventions in UtilityFunctions.ts
- File organization best practices in TooLongFile.ts
- Indentation consistency in InconsistentIndentation.ts

All examples use standard patterns familiar to TypeScript developers.

---

## Next Steps

1. Copy files to a temporary directory for testing
2. Run Dictator against the sandbox
3. Compare output to TESTING_CHECKLIST.md
4. Document any discrepancies in VIOLATION_EXAMPLES.md
5. Test auto-fix functionality (if available)
6. Validate corrected files match expected output
