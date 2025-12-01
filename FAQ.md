# FAQ

## General

### Why is it called "Dictator"?

Because rules are meant to be broken. Decrees are not.

Your linter has 500 rules. You disabled 200. You added 50 inline ignores. You have a meeting next week to discuss which rules to enable.

Dictator doesn't negotiate. File ends with newline or it fails. Tabs or spaces, pick one. No meetings required.

### Is the name offensive?

It's a linter, not a political statement. If a software tool's name prevents you from using it, you have bigger problems.

### Is this a joke project?

The satire is real. The code is also real. Both can be true.

### Why not just configure RuboCop/ESLint better?

You can. But:
1. Those tools are slow (AST parsing, type checking)
2. Dictator runs first, fails fast on structural issues
3. Why wait 45 seconds for RuboCop to tell you there's trailing whitespace?

### Who is this for?

- Teams with LLMs generating code (Claude, Copilot, Codex)
- Monorepos with multiple languages
- CI pipelines that need fast feedback
- Developers who hate waiting
- Anyone with years of accumulated notes, docs, and code

### The 4.5 Million Lines Problem

Real story: I have 4.5 million lines of code, notes, and documentation in my workspace.

One day I opened a file that looked empty in vim. Deleted it. It was 4.5kb of "empty" - 500 lines of whitespace followed by important notes I wrote a year ago. I had left space for "an introduction I'll write later" (I never did).

With Dictator watching:
- `supreme/blank-line-whitespace` - catches those 500 empty lines
- `supreme/trailing-whitespace` - finds the mess
- Works on `.md`, `.txt`, notes, everything

Dictator isn't just for code. It's for anyone who accumulates files faster than they organize them.

### Who is this NOT for?

- People who need a meeting to decide on tab width
- Teams that disable linter rules by majority vote
- Anyone who thinks "it compiles" means "it's good"

## Technical

### Why Rust?

Fast. Safe. Single binary. No runtime dependencies.

### Why WASM for decrees?

- Sandboxed (decrees can't corrupt the Regime)
- Portable (write once, run anywhere)
- Distributable (share company decrees as .wasm files)

### Does Dictator replace RuboCop/ESLint?

No. Dictator is a pre-linter. It runs before your expensive linters:

```
Code → Dictator (2s) → RuboCop (45s) → Deploy
```

Catch structural violations instantly. Run quality linters on clean code.

### What's the performance like?

Milliseconds for structural checks. We don't parse ASTs. We count lines, check whitespace, validate ordering.

### Can I write custom decrees?

Yes. See [DECREES.md](DECREES.md). Build as WASM, load via config.

Every decree includes metadata for ABI compatibility checking (versioning, description, capabilities, supported file extensions).

### What happens if my decree's ABI version doesn't match?

Dictator validates decree ABI compatibility at load time. If your decree was built against a different version of `dictator-decree-abi`, it will fail to load with a clear error:

```
Error: Decree 'my-decree' from /path/to/decree.wasm:
ABI version mismatch: host 0.1.0, decree 0.2.0

This decree was built against a newer API version. Please:
- Rebuild the decree against dictator-decree-abi 0.1.x, or
- Upgrade dictator to support ABI 0.2.0
```

**Pre-1.0 versioning:** Decrees must have exact major.minor version match (0.1.x works with 0.1.y, but 0.1.x doesn't work with 0.2.0).

### What's a "Regime"?

The Regime is the enforcement engine. It owns decree instances and runs them over your sources:

```rust
let mut regime = Regime::new();
regime.add_decree(dictator_supreme::init_decree());
regime.add_decree(dictator_typescript::init_decree());
regime.enforce(&sources)?;
```

Decrees don't add capabilities - they enforce compliance. The Regime doesn't negotiate.

### What's the Dictator Dialect?

The standard output format for Dictator-compatible linters. See [DIALECT.md](DIALECT.md).

Linters that output this format = first-class citizens.
Linters that need parsing = second-class citizens.

### What does `enforced` mean?

The `enforced` field in diagnostics indicates whether Dictator auto-fixed the violation:

| Value | Meaning | Display |
|-------|---------|---------|
| `true` | Dictator fixed it | 🔧 |
| `false` | User must comply | ❌ |

For external linters (RuboCop, ESLint, Clippy, Ruff), `enforced` is determined by whether the linter can auto-fix:
- RuboCop: `correctable` field
- ESLint: `fix` object presence
- Clippy: `MachineApplicable` suggestion
- Ruff: `fix.applicability == "safe"`

## MCP / AI Integration

### What's MCP?

Model Context Protocol. It allows Claude and other AI assistants to take wisdom from the Dictator on how to reign over files with an iron fist.

The Dictator teaches. The AI learns. The files comply.

### Why would an AI need a linter?

LLMs generate structural chaos. 300 files, all compile, all have wrong structure:
- Random frontmatter order
- Private methods before public
- 2000-line files
- Trailing whitespace everywhere

Dictator catches this instantly. The LLM fixes it. Then the real linters run.

### Does Dictator "align" the AI?

No. Dictator aligns the *files* the AI produces. The AI is already aligned (presumably).

## Philosophy

### Why "decrees" instead of "rules"?

Rules are meant to be broken. Ask any developer with `# rubocop:disable` in their code.

Here's how contraband enters your codebase:

1. You have 200 folders. Someone adds `# rubocop:disable all` in `app/action_services/`
2. RuboCop reports: 0 errors. Clean bill of health.
3. Months pass. The folder grows. Nobody notices.
4. One day you realize the folder belongs in `lib/`, not `app/`
5. You move it. RuboCop reads it fresh. No disable comment in `.rubocop.yml` for this path.
6. **40,000+ errors.** The folder was accumulating violations the whole time.
7. You can't fix 40,000 errors. You remove RuboCop.

This is how linters die. One disable at a time.

Dictator doesn't read comments. `# dictator:disable` does nothing. There is no `.dictator_todo.yml` for "temporary" exceptions that become permanent.

Decrees are absolute. The Dictator does not negotiate.

### Why is Kim Jong Rails GPL licensed?

Glorious People Licence. For the people, by the people, enforced by the Dictator.

### Is this project maintained?

Yes. The Dictator does not abandon their subjects.

### Why no decree for C/Haskell/Crystal/Elixir/Java/etc?

I'm not trying to impress anyone.

I could generate decrees for 50 languages with Claude/Gemini in an afternoon. But I don't use those languages daily. I haven't studied their conventions deeply.

When I started learning Go and Rust, I put tests in a `test/` folder, far from the source files. That's how Ruby does it - I was bringing my Ruby habits. It worked. But it was alien to those ecosystems. I was doing it wrong without knowing.

I don't want to upset real language users with decrees written by someone who doesn't live in their ecosystem. That's how you get rules that technically work but feel wrong.

The decrees included are for languages I use. When I learn a new language properly, its decree will follow.

If you live in a language ecosystem and want to contribute a decree - welcome. Just bring elegance and common sense, not your personal vendetta against semicolons.

### Why aren't all your rules in the decrees?

My personal rules are too specific to my workflow. Decrees should be universal conventions, not my opinions.

When something becomes widely accepted - tabs in Go, 2-space indent in Ruby, imports order in Python - it can become a decree. My "I like blank lines before returns" stays in my `.editorconfig`.

### Can I contribute?

Yes. Submit a PR. If it passes the decrees, it will be reviewed.

### What if I disagree with a decree?

You have options:
1. Fork the decree
2. Convince the decree maintainer
3. Comply

Option 3 is fastest.

---

**Still have questions?** Open an issue.
