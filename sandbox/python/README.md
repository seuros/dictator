# Python Linter Test Files

Test fixtures for Dictator Python linter with intentional structural violations.

## Files

### 1. `trailing_whitespace.py` (28 lines)
- **Violation**: Trailing whitespace on 6 lines
- **Lines affected**: 7, 9, 12, 19, 23, 28
- **Details**: Multiple functions and a class with trailing spaces at end of lines
- **Syntactically valid**: Yes

### 2. `mixed_indentation.py` (35 lines)
- **Violation**: Mixed tabs and spaces
- **Lines affected**: 14-17 (tabs), 29-32 (tabs in class methods)
- **Details**: Some functions/methods use tabs while others use spaces
- **Python severity**: Critical - affects `IndentationError` detection
- **Syntactically valid**: No - will raise `IndentationError` if executed

### 3. `wrong_import_order.py` (38 lines)
- **Violation**: Incorrect import statement ordering
- **Details**: 
  - `import requests` (3rd-party) before `import os` (stdlib)
  - `from typing import ...` before `import json`
  - `from collections import ...` after standard library imports
- **Standard order**: `__future__` → stdlib → 3rd-party → local
- **Syntactically valid**: Yes

### 4. `long_file_mixed_endings.py` (450 lines)
- **Violations**: Multiple structural issues
  - File length: 450 lines (exceeds typical threshold of 300-400)
  - Mixed line endings: Both CRLF (`\r\n`) and LF (`\n`)
  - No final newline: (Original version - note: current version has newline)
- **Details**: Generated functions and methods with intentional line ending mixing
- **Syntactically valid**: Yes

### 5. `no_final_newline.py` (26 lines)
- **Violation**: Missing final newline at EOF
- **Last line**: `print(process())` (no `\n`)
- **Details**: Valid Python code but doesn't end with newline character
- **Syntactically valid**: Yes

## Testing Checklist

- [ ] Dictator detects trailing whitespace violations
- [ ] Dictator flags mixed indentation (tabs/spaces)
- [ ] Dictator enforces import ordering (PEP 8)
- [ ] Dictator detects excessive file length
- [ ] Dictator detects mixed line endings (CRLF/LF)
- [ ] Dictator detects missing final newline
- [ ] All violations reported with correct line numbers
- [ ] Correct file paths shown in output

## Expected Severity Levels

- **Trailing whitespace**: Low/Medium (style)
- **Mixed indentation**: High (Python syntax sensitive)
- **Wrong import order**: Low (style/convention)
- **File too long**: Medium (maintainability)
- **Mixed line endings**: Medium (portability)
- **No final newline**: Low (POSIX compliance)
