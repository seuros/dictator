# The Dictator

**Fast structural enforcement, before the linters.**

Dictator is a pre‑linter structural gatekeeper for your codebase. It doesn't replace RuboCop, ESLint, or Clippy — it runs *before* them. While those tools analyze code quality, Dictator enforces the boundaries: file structure, naming conventions, ordering, and basic hygiene.

Canonical lore (Timeline 7) says Dictator was manifested in Rust 60 and then backported to Rust 1.91. In this timeline, it is a conventional Rust crate that you build and run with a modern stable toolchain.

Think of it as border control for your codebase: everything must satisfy basic structural discipline before the expensive tools run.

## TL;DR

- Run **fast, structural checks** before slow linters.
- Enforce **file/line limits, naming, ordering, and hygiene**.
- Drive **LLM workflows**, **CI**, and **monorepos** with one config: `.dictate.toml`.
- Extend via **WASM decrees** and an **MCP server** for AI assistants.

## Installation

### Binary Release (Recommended)

Download and install a pre-built binary for your platform:

```bash
curl -fsSL https://raw.githubusercontent.com/seuros/dictator/master/scripts/install.sh | bash
```

This installs `dictator` to `~/.local/bin` by default. Make sure this directory is on your PATH:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Installation options:**
- `--prefix <dir>` — Install to a custom directory
- `--version <tag>` — Install a specific release version
- `--help` — Show all options

Example:
```bash
curl -fsSL https://raw.githubusercontent.com/seuros/dictator/master/scripts/install.sh | bash -- --prefix ~/.cargo/bin
```

### Build from Source

Requires Rust 1.91+:

```bash
cargo install --git https://github.com/seuros/dictator
```

Or from this repository:

```bash
cargo install --path crates/dictator
```

### Runtime Requirements

- **Linux**: glibc 2.31+ (most modern distributions)
- **macOS**: 11.0+ (Intel or Apple Silicon)

## The Problem

**Expensive linters are slow.** Running RuboCop on a large Rails codebase takes minutes. ESLint on a monorepo crawls.

**LLMs generate structural chaos.** Claude creates 300 files. All compile. All have wrong structure: inconsistent naming, files with 2000 lines, frontmatter fields in random order, private methods in wrong positions.

**You need fast boundary checks first.** Catch structural violations in milliseconds, not minutes. Then run the expensive linters on code that already passes basic discipline.

## What Dictator Enforces

**File boundaries:**
- Maximum line count (ignoring comments)
- Trailing whitespace, tabs vs spaces
- Final newline presence
- Line ending consistency (LF vs CRLF)

**Naming conventions:**
- Folder names (kebab-case, snake_case, etc.)
- File names matching patterns
- Function/class name conventions

**Ordering discipline:**
- Frontmatter field order (slug after title, pubDate in position 2)
- Method visibility sections (public → protected → private)
- Import/require statement grouping
- YAML/TOML key ordering

**Basic hygiene:**
- No emojis in source code (structural noise)
- Copyright/license header presence (structural requirement)
- Comment formatting (`#foo` → `# foo`)

**What Dictator does NOT do:**
- Code quality analysis (use RuboCop, ESLint, Clippy for that)
- Context-dependent enforcement (focus markers, risky patterns—might be legitimate)
- Type checking (use your language's type system)
- Complexity metrics (use dedicated tools)
- Performance analysis

Dictator checks structure. The VIPs (RuboCop, ESLint) check quality and context.

## Why This Matters

**Speed.** Dictator runs structural checks in milliseconds. Catch obvious violations instantly without waiting for heavy linters.

**LLM workflows.** When Claude generates 300 files, Dictator validates structure immediately:
```
Claude generates code → Dictator checks structure → Fix → RuboCop checks quality → Done
```

**CI optimization.** Fail fast on structural violations before expensive linter passes:
```
git push → Dictator (2ms) → ❌ File too long → Fix locally
git push → Dictator (2ms) → ✓ → RuboCop (45s) → ✓ → Deploy
```

**Monorepo enforcement.** One binary, one config, all languages. Consistent structural rules across Ruby services, TS frontends, YAML configs.

## Why Decrees, Not Rules?

Rules are meant to be broken. That's why you have 1000 linters with 10000 rules and everyone disables half of them.

Decrees are absolute. The Dictator does not negotiate. Your file ends with a newline or it doesn't pass. Your methods are ordered correctly or they aren't. No "warn", no "suggestion", no "consider maybe perhaps".

This is structural discipline, not style advice.

## Example: Frontmatter Order

Your blog posts need consistent frontmatter. LLMs swap field order randomly:

```yaml
---
pubDate: 2025-12-01
title: "My Post"
slug: my-post
---
```

Dictator enforces order:
```yaml
---
title: "My Post"
slug: my-post
pubDate: 2025-12-01
---
```

Compiles either way. Dictator doesn't tolerate the first. Structure is not negotiable.

## Example: Ruby Comment Hygiene

LLMs generate comments without proper spacing. Dictator catches it:

```ruby
#bad comment    # ❌ Missing space after #
# good comment  # ✓ Correct
```

Dictator auto-fixes `#bad` → `# bad`. RuboCop checks style. Dictator checks structure.

## Architecture

```
.dictate.toml (decree configuration)
    ↓
dictator (Rust CLI, single binary)
    ↓
dictator-core (wasmtime, parallel execution)
    ↓
dictator-decree-abi (shared ABI: Plugin/Diagnostic types)
    ↓
WASM decrees (decree.supreme, decree.ruby, decree.golang, ...)
    ↓
Diagnostics (JSON/SARIF/stdout)
```

**Decree-driven enforcement.** `.dictate.toml` declares which decrees are active. Dictator loads corresponding WASM components and runs them in parallel.

**All WASM.** Every decree is a WASM component:
- **`decree.supreme`**: Universal structure (spacing, whitespace, line endings)
- **`decree.<language>`**: Language-specific structure (method ordering, naming, conventions)

**Why WASM:**
- Sandboxed (decrees can't corrupt Dictator)
- Distributable (share company-specific decrees)
- Extensible (add new languages without rebuilding)
- Isolated (no dependency conflicts)

**Decree Versioning.** Every decree exports metadata including ABI version:
- Dictator validates decree compatibility at load time
- Incompatible decrees fail fast with clear errors
- Pre-1.0: decrees built with different ABI versions won't load
- Future-proof: supports API evolution (fix(), streaming, config at lint-time)

**Speed first.** No heavy AST parsing. Pattern matching, line counting, regex. Fast enough for watch mode.

## How Dictator Works (You Press Enter, It Starts Yelling)

At a high level:

1. **You point Dictator at some paths.**
   `dictator lint .`, `dictator watch sandbox/`, or whatever mess your LLM or your team just hallucinated.

2. **It reads `.dictate.toml`.**
   This is the decree book. It decides:
   - which decrees are enabled (`decree.supreme`, `decree.ruby`, `decree.typescript`, …)
   - what the limits are (max lines, allowed line endings, naming rules, etc.)

3. **It walks the filesystem.**
   Dictator does a fast pass over the files you pointed at: no ASTs, no type-checking, just “what files exist, what are their extensions, how big are they”.

4. **It assigns each file to decrees.**
   - Every file is judged by **`decree.supreme`** (whitespace, line endings, length, final newline).
   - Language files get additional judges: `*.rb` → `decree.ruby`, `*.ts` → `decree.typescript`, `*.go` → `decree.golang`, etc.

5. **It runs all decrees in parallel as WASM.**
   Each decree is a sandboxed WASM component. Dictator:
   - feeds it the file contents + config
   - waits for diagnostics (violations) to come back
   - never lets decrees touch your filesystem or spawn surprise subprocesses

6. **It surfaces diagnostics.**
   Dictator reports:
   - file + line + column
   - which decree complained
   - a short, rude description of what you did wrong (trailing whitespace, file too long, wrong visibility order, etc.)

7. **(Optional) It fixes what it can.**
   In auto-fix modes, Dictator will happily:
   - strip trailing whitespace
   - normalize line endings
   - add missing final newlines
   and then leave the harder, semantic work to your “real” linters.

The entire pipeline is “cheap first, expensive later”: Dictator slaps your structure into shape, then your quality linters and type-checkers show up once the room is already clean.

## Usage

Dictator reads `.dictate.toml`, loads the configured decrees (WASM components), and runs enforcement.

```bash
# Lint files (read-only, reports violations)
dictator lint src/
dictator stalint src/          # alias

# Fix structural issues (trailing whitespace, CRLF→LF, final newline)
dictator dictate src/
dictator kjr src/              # alias

# Watch mode (re-check on every save)
dictator watch .

# Specify custom config
dictator --config .dictate.dev.toml lint src/
```

| Command | Alias | Mode | Description |
|---------|-------|------|-------------|
| `lint` | `stalint` | Read-only | Reports violations without modifying files |
| `dictate` | `kjr` | Destructive | Fixes whitespace, line endings, final newline |
| `watch` | - | Read-only | Monitors files and reports on change |

## Watch Mode: Real-Time Enforcement

Watch mode monitors file changes and validates instantly:

```bash
dictator watch .
```

**LLM workflow:**
```
You: "Claude, generate user auth module"
Claude: *creates 15 files*
dictator stalint .   → ❌ auth_helper.rb has trailing whitespace
dictator dictate .   → 🔧 Fixed 3 files
dictator stalint .   → ✓ All structural checks pass
RuboCop: *runs expensive quality checks*
```

**Human workflow:**
```
Save file → dictator stalint (50ms) → dictator dictate → Done
```

Dictator reports. You fix. Pre-commit hooks can auto-fix if you want automation.

## Configuration: Decrees

`.dictate.toml`:
```toml
[decree.supreme]
# Universal structural rules (all files, all languages)
trailing_whitespace = "deny"
tabs_vs_spaces = "spaces"
tab_width = 2
final_newline = "require"
line_endings = "lf"
max_line_length = 120

[decree.ruby]
# Ruby-specific structural enforcement
max_lines = 300

[decree.golang]
# Go uses tabs, not spaces — overrides decree.supreme
tabs_vs_spaces = "tabs"
max_lines = 500

[decree.frontmatter]
# Frontmatter ordering (Markdown, Astro, etc.)
order = ["title", "slug", "pubDate", "tags"]
required = ["title", "slug"]
```

**Language overrides.** Language decrees can override supreme settings. Go files use tabs even when supreme says spaces. The override applies per-file based on extension.

**Decrees are WASM components.** Each decree enforces structural boundaries for its domain. `decree.supreme` applies universally. Language decrees handle specific conventions.

## Current Decrees

**`decree.supreme`** (universal):
- Trailing whitespace detection
- Tab vs space enforcement
- Missing final newline
- Line ending consistency (LF/CRLF)
- Max line length

**`decree.ruby`**:
- File line count limits (ignoring comments/blank lines)
- Comment spacing (`#foo` → `# foo`)
- Tab detection (Ruby uses spaces)
- Blank line whitespace cleanup

More decrees coming. Each language gets its own WASM decree for structural enforcement.

## Roadmap

There is no roadmap. The Dictator does not make promises.

Contributions are welcome if accompanied by:
- **Concrete benchmarks** — prove your decree is fast
- **Real usage** — show it solves an actual problem you have

## Building Custom WASM Decrees

See **`decree.kjr`** (Kim Jong Rails) in [DECREES.md](DECREES.md) for a complete example of building a custom decree using only the `dictator-decree-abi` crate.

The KJR decree demonstrates:
- Implementing the `Plugin` trait
- Emitting `Diagnostic` violations
- Building as a WASM component
- Loading via `.dictate.toml` config

## MCP Server

In Timeline 7, everything runs on KIMFS (Kim File System). Files cannot be structurally unsound — the filesystem itself rejects malformed structure at write time. Trailing whitespace? Denied. Wrong line endings? Denied. Methods in wrong order? Believe it or not, denied. Dictator is not a linter there, it's a fundamental law of physics.

In this timeline, files are not sentient and Dictator is a normal CLI binary. It only starts enforcing structure once it sees a configuration file:

- **`.dictate.toml`** — not YAML, not XML. TOML is the contract.

Once configuration exists, Dictator has two operational modes:

- **Watch and snitch** — `dictator watch` monitors your files and reports structural violations as you edit.
- **Surprise inquisition** — `dictator lint` walks your tree and reports every violation in a single pass.

AI coding assistants like Claude Code and OpenAI Codex can use Dictator not to “align” the LLM itself, but to **align the files** the LLM produces. Through MCP, they get two tools:

- **`stalint`** (Static Lint). Despite the name, it doesn’t “lint” in the classic sense — it just **reports** structural violations: trailing whitespace, line endings, file size, etc. Read-only. No surprises.
- **`dictator`**. This one actually does things:
  - In **`kimjongrails`** mode, it fixes native structural errors (LF/CRLF, trailing spaces, missing final newlines, etc.).
  - In **`supremecourt`** mode, it escalates to whatever primitive linters you already trust (RuboCop, ESLint, Prettier, Ruff, …) as defined in `.dictate.toml`.

From the AI’s point of view, Dictator is the one calling the shots: external linters do the heavy lifting, Dictator orchestrates them, and then takes the credit.

Dictator includes an MCP (Model Context Protocol) server so these tools are discoverable and callable from compatible AI coding assistants.

### Installation

Add to your Claude Code MCP configuration (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "dictator": {
      "command": "/path/to/dictator"
    }
  }
}
```

MCP mode is auto-detected when stdin is a pipe and no CLI arguments are provided.

### Available Tools

| Tool | Description | Mode | Availability |
|------|-------------|------|--------------|
| `stalint` | Check files for structural violations (trailing whitespace, tabs/spaces, line endings, file size). Returns diagnostics without modifying files. Can check any path. | Read-only | Always |
| `dictator` | Auto-fix structural issues. Mode `kimjongrails` fixes whitespace/newlines. Mode `supremecourt` runs configured external linters from `.dictate.toml`. | Destructive | Git repos only |
| `stalint_watch` | Watch paths for file changes. Runs stalint every 60s when changes detected. Restricted to cwd. | Read-only | Always |

**Tool modes are dynamic:**
- `kimjongrails`: Always available (basic structural fixes)
- `supremecourt`: Only available if decrees have configured linters (e.g., `decree.ruby.linter.command = "rubocop"`)

### Example Usage

From your AI assistant:
```
Check sandbox/ for structural violations
```

The assistant will call `stalint` and report violations with file, line, column, rule, and message.

To auto-fix:
```
Fix structural issues in sandbox/
```

The assistant will call `dictator` which fixes trailing whitespace, missing final newlines, and CRLF→LF conversions.

**Using `supremecourt` mode:**
If you have configured linters in `.dictate.toml`, the assistant can run them via `supremecourt` mode:
```
Fix structural issues in sandbox/ using supremecourt mode
```

The MCP server will:
1. Detect file types in the provided paths (`.rb` → ruby, `.ts` → typescript, etc.)
2. Check which decrees have configured linters
3. Execute configured linters for detected file types
4. Return combined output

### Safety Features

**Multi-layer protection against destructive operations:**

1. **Git repository requirement**: `dictator` tool only exposed when `.git` directory exists
2. **Working directory boundary**: Destructive tools (`dictator`, `stalint_watch`) reject paths outside cwd
3. **Sandbox mode support**: Dictator advertises `codex/sandbox-state` capability and hides destructive tools in read-only mode
4. **Dynamic mode detection**: `supremecourt` mode only available if external linters are installed

**Examples:**
- Run from `/tmp` (no git) → LLM only sees `stalint` (read-only)
- Run from project (has git) → LLM sees `dictator` but it rejects `/home` or `/etc`
- No rubocop/eslint → `supremecourt` mode hidden from LLM

**Note:** As of 2025, some MCP clients don't send sandbox notifications. See Claude Code issues [#3315](https://github.com/anthropics/claude-code/issues/3315), [#3174](https://github.com/anthropics/claude-code/issues/3174), [#3141](https://github.com/anthropics/claude-code/issues/3141) for related discussion.

### Configuring External Linters

The MCP server reads linter configurations from `.dictate.toml`:

```toml
[decree.ruby.linter]
command = "rubocop"

[decree.typescript.linter]
command = "eslint"

[decree.python.linter]
command = "ruff"

[decree.golang.linter]
command = "gofmt"
```

**Dictator controls the args.** You only specify the command. Dictator adds the appropriate flags for auto-fix and JSON output parsing:
- `rubocop` → `-A --format json`
- `eslint` → `--fix --format json`
- `ruff` → `check --fix --output-format json`
- `gofmt` → `-w` (lists changed files, then fixes)
- `clippy` → `--fix --allow-dirty --message-format json`

**How it works:**
- MCP server detects file types in provided paths
- Maps extensions to decree names (`.rb` → `ruby`, `.ts`/`.js` → `typescript`, `.py` → `python`, `.go` → `golang`, etc.)
- Executes configured linter with Dictator-controlled args
- Parses JSON output to unified diagnostics (🔧 fixed, ⚠️ warning, ❌ error)
- `supremecourt` mode only appears in tool list if at least one decree has a configured linter installed

**Security:** Linters run as subprocesses with provided file paths as arguments. User is responsible for ensuring configured commands are safe.

## Use Cases

**Pre-linter CI stage:**
```yaml
- name: Structural checks (fast)
  run: dictator lint .  # Fails fast if structure is wrong
- name: Quality checks (slow)
  run: bundle exec rubocop
```

**Pre-commit workflow:**
```yaml
# .pre-commit-config.yaml
- repo: local
  hooks:
    - id: dictator
      name: Structural enforcement
      entry: dictator lint
      language: system
      pass_filenames: true
```

**LLM code generation guard:**
```
AI generates code → dictator stalint → dictator dictate → Quality linters run
```

**Monorepo boundary enforcement:**
One config, all languages. Ruby services, TS apps, YAML configs—same structural rules.

**Development speed:**
Instant feedback on saves. `dictator stalint` reports in milliseconds. `dictator dictate` fixes them.

## License

MIT

---

**Dictator**: Snitches on your structure. Takes all the credit.
