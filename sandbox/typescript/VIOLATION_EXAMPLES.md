# TypeScript Violation Examples

Detailed examples of violations found in each test file.

## BadReactComponent.tsx

### Violation 1: Mixed Tabs and Spaces
```typescript
interface Props {
  title: string;
→	name: string;              // LINE 10: Starts with TAB instead of spaces
  disabled?: boolean;
}
```
The tab character (\t) is inconsistent with the space indentation on lines 9 and 11.

### Violation 2: Inconsistent Indentation in Function
```typescript
export const BadReactComponent: FC<Props> = ({ title, name, disabled = false }) => {
→	const [loading, setLoading] = React.useState(false);  // TAB
  const [data, setData] = useState<string | null>(null);  // SPACES
→	const [error, setError] = useState<Error | null>(null); // TAB
```
Lines 15, 16, 17 mix tabs and spaces for the same indentation level.

### Violation 3: Multiple Tab Occurrences
- Line 15: `→	const [loading, ...`
- Line 17: `→	const [error, ...`
- Line 19: `→	useEffect(() => {`
- Line 22: `→→		const fetchData = async () => {`
- Line 24: `→→			try {`
- Line 26: `→→		  setError(err as Error);`
- Line 28: `→→		finally {`
- Line 31: `→→	};`
- Line 38: `→	const handleClick = () => {`
- Line 44: `→	<p>{name}</p>`
- Line 52: `→	{data && <div className="data">{data}</div>}`

### Violation 4: Missing Final Newline
File ends at line 57 without a final newline character:
```
56→};
57→
58→export default BadReactComponent;
    ↑ (EOF - no newline after semicolon)
```

---

## UtilityFunctions.ts

### Violation 1: Wrong Import Ordering
Imports are randomly grouped instead of following convention:

Current order (WRONG):
```typescript
// Line 1-3: Third-party (date-fns, lodash, uuid)
import { format } from 'date-fns'
import lodash from 'lodash'
import { v4 as uuidv4 } from 'uuid'

// Line 4-6: System imports and local imports mixed
import * as fs from 'fs'
import type { Logger } from './types'
import { config } from './config'

// Line 7-8: More system imports
import crypto from 'crypto'
import * as path from 'path'

// Line 9: Local imports
import { validateInput } from './validators'

// Line 10: Third-party axios (should be with others at top)
import axios from 'axios'
```

Expected order:
```typescript
// System imports first
import * as fs from 'fs'
import * as path from 'path'
import crypto from 'crypto'

// Third-party imports second
import { format } from 'date-fns'
import lodash from 'lodash'
import { v4 as uuidv4 } from 'uuid'
import axios from 'axios'

// Local imports last
import type { Logger } from './types'
import { config } from './config'
import { validateInput } from './validators'
```

### Violation 2: Missing Final Newline
File ends at line 151 without final newline:
```
149→  return array.reduce(
150→    (acc, item) => {
151→      acc[predicate(item) ? 0 : 1].push(item);
152→      return acc;
153→    },
154→    [[], []] as [T[], T[]]
155→  );
156→}
   ↑ (EOF - no newline)
```

---

## TooLongFile.ts

### Violation 1: File Exceeds Length Limit
File has **394 lines**, significantly exceeding recommended maximum of 250-300 lines.

Contents:
- Lines 1-50: Imports and constants
- Lines 51-88: UserRepository class
- Lines 89-126: ProductRepository class
- Lines 127-164: OrderRepository class
- Lines 165-236: ApiService class (Angular injectable)
- Lines 237-273: EventBus class
- Lines 274-306: StateManager class
- Lines 307-343: RequestQueue class
- Lines 344-394: Utility functions and exports

### Violation 2: Mixed Line Endings
File contains both Unix (LF) and Windows (CRLF) line endings:
- Most lines use LF (\n)
- Some lines use CRLF (\r\n)
- Creates git diff noise when viewed

Example detection:
```bash
file TooLongFile.ts
# Output: C++ source text, ASCII text, with CRLF, LF line terminators
```

### Violation 3: Missing Final Newline
File ends at line 394 without final newline:
```
392→export const utilities = {
393→  generateId,
394→  delay,
395→  retryOperation,
396→  memoize,
397→};
   ↑ (EOF - no newline)
```

---

## InconsistentIndentation.ts

### Violation 1: Severely Inconsistent Indentation

Lines with 1-2 spaces (minimal indentation):
```typescript
// Line 32-33
static isString(value: any): value is string {
 return typeof value === 'string';  // Only 1 space!
}
```

Lines with tab indentation:
```typescript
// Line 39-41
→	static isBoolean(value: any): value is boolean {
    return typeof value === 'boolean';
}
```

Lines with excessive spacing:
```typescript
// Line 47
  static isArray<T = any>(value: any): value is T[] {
       return Array.isArray(value);  // 7 spaces!
  }
```

Lines with mixed tabs and spaces:
```typescript
// Line 49
static isObject(value: any): value is Record<string, any> {
 	   return value !== null && typeof value === 'object' && !Array.isArray(value);
    // ↑ 1 space + 1 tab + 3 spaces = mixed indentation
}
```

### Violation 2: Variable Indentation Within Classes

StringUtils class methods have inconsistent indentation:
```typescript
export class StringUtils {
  static capitalize(str: string): string {
    if (StringUtils.isEmpty(str)) return str;
      return str.charAt(0).toUpperCase() + str.slice(1);
    // ↑ Extra 2 spaces, not aligned with previous line
  }

  static camelCase(str: string): string {
   const parts = str.split(/[-_\s]+/);
   // ↑ Only 1 space indent
     return parts
      .map((part, idx) => idx === 0 ? part.toLowerCase() : this.capitalize(part))
       .join('');
    // ↑ Varying spaces: 1, 1, 2, 3
  }
}
```

### Violation 3: Missing Final Newline
File ends at line 208 without final newline:
```typescript
206→export const Utilities = {
207→  StringUtils,
208→  ArrayUtils,
209→  ObjectUtils,
210→};
   ↑ (EOF - no newline)
```

### Violation 4: Indentation in ArrayUtils Methods
```typescript
export class ArrayUtils {
  static unique<T>(array: T[]): T[] {
  return [...new Set(array)];  // No indent! (line 113)
  }

  static flatten<T>(array: any[]): T[] {
     return array.reduce((acc, val) => acc.concat(val), []);  // 5 spaces
  }

  static chunk<T>(array: T[], size: number): T[][] {
    const chunks: T[][] = [];
      for (let i = 0; i < array.length; i += size) {  // Extra indent
      chunks.push(array.slice(i, i + size));  // Misaligned
    }
    return chunks;
  }
}
```

---

## Summary Table

| File | Tab Issues | Space Issues | Import Issues | Length | Line Ending | Final NL |
|------|-----------|--------------|---------------|--------|-------------|----------|
| BadReactComponent.tsx | 18 instances | Multiple | N/A | 56 lines | LF | Missing |
| UtilityFunctions.ts | None | N/A | Mixed randomly | 151 lines | LF | Missing |
| TooLongFile.ts | None | N/A | N/A | 394 lines | Mixed CRLF | Missing |
| InconsistentIndentation.ts | 3 instances | 1-8 spaces vary | N/A | 208 lines | LF | Missing |

**Total Violations: 40+**

Each violation requires Dictator to normalize:
- Replace tabs with spaces (configurable, typically 2-4 spaces)
- Trim trailing whitespace
- Remove extra/inconsistent indentation
- Add final newlines
- Normalize line endings
- Reorder imports by category
- Consider splitting long files
