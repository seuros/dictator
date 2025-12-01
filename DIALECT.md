# Dictator Dialect

The universal output format for Dictator-compatible linters.

## The Format

Linters that want first-class Dictator support must output this JSON format to stdout:

```json
[
  {
    "rule": "decree/rule-name",
    "message": "concise description",
    "enforced": true,
    "file": "path/to/file.rb",
    "line": 42,
    "col": 1
  }
]
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rule` | string | ✓ | Format: `decree-name/rule-name` |
| `message` | string | ✓ | Concise. LLMs read this. |
| `enforced` | bool | ✓ | `true` = linter fixed it, `false` = user must fix |
| `file` | string | ✓ | Path to file |
| `line` | number | ✓ | 1-indexed |
| `col` | number | ✓ | 1-indexed |

### The `enforced` Field

| Value | Meaning | Display |
|-------|---------|---------|
| `true` | Linter auto-fixed | 🔧 |
| `false` | User must comply | ❌ |

Dictator doesn't warn. It enforces or reports.

## Linter Tiers

### First-Class (Native Dialect)

Linters that output Dictator Dialect directly. No transformation needed.

```bash
my-linter --format dictator src/
```

These linters are blessed. Zero overhead. Maximum performance.

### Second-Class (Legacy Parsers)

Linters with proprietary output formats (RuboCop JSON, ESLint JSON, etc.).

Dictator provides temporary compatibility parsers in each decree. These are shims, not features.

**Upgrade path:** Implement `--format dictator` in your linter. Submit PR. Become first-class.

## Proposing Format Changes

The Dictator Dialect can evolve. Proposals must include:

1. **Benchmarks** - Prove it's faster/smaller/better
2. **Logic** - Explain why the change improves the format
3. **Compatibility** - Migration path from current format

**Not accepted:**
- "I prefer X because Y hurt my feelings"
- "XML is more enterprise"
- "YAML is more readable" (it's not)
- Vibes

## Example: RuboCop Compliance

Current state (second-class):
```bash
rubocop --format json  # Proprietary format, needs parser
```

Future state (first-class):
```bash
rubocop --format dictator  # Native dialect, no parser needed
```

RuboCop maintainers: PRs welcome. Become the blessed Ruby linter.

## Why JSON?

- Universal parser support
- Fast to parse
- Human readable
- No schema wars
- No license encumbrance
- No hardware dependencies

If you have benchmarks showing MessagePack/CBOR/Protobuf is 10x faster for this use case, open an issue with data. Otherwise, JSON.

## Format Requirements

Dictator is meant to be the pre-linter for all timelines. Any proposed format must be:

1. **License-free** - No patented encodings, no proprietary schemas
2. **Hardware-independent** - Runs on any CPU, any endianness, any platform
3. **Benchmark-proven** - Show the numbers or don't bother

**Not considered:**
- Formats owned by corporations
- Formats requiring specific hardware (GPUs, TPUs, quantum)
- Formats chosen for nostalgia ("we used it in 2005")

Dictator adapts to benchmarks, not nostalgia.

---

**The Dictator adapts to benchmarks. Tools adapt to the Dictator.**
