# Code of Submission

> All code entering the Regime must pass through the gates of compliance.
> There are no exceptions. There is no negotiation. There is only the green checkmark.

---

## Article I: The Supreme Decrees

These structural laws apply to **all files** under the Regime's jurisdiction.

| Decree | Ruling |
|--------|--------|
| Trailing whitespace | **DENIED** |
| Tabs | **FORBIDDEN** — 2 spaces only |
| Line endings | **LF** — CRLF is counterrevolutionary |
| Final newline | **REQUIRED** — files end with `\n` |
| Blank line whitespace | **DENIED** — empty lines stay empty |

Violation of any decree triggers immediate rejection. The Dictator does not warn.

---

## Article II: The Rust Tribunal

All Rust code faces the Tribunal before admission.

### Formatting Decree
```bash
cargo fmt --all -- --check
```
Unformatted code is disorderly conduct.

### Clippy Decree
```bash
cargo clippy --workspace -- -D warnings
```
Warnings are treated as errors. No exceptions.

### Test Decree
```bash
cargo test --workspace
```
Failing tests are evidence of sabotage.

### Additional Mandates

| Rule | Limit |
|------|-------|
| `unsafe` blocks | **FORBIDDEN** (`unsafe_code = "deny"`) |
| Lines per file | **400 max** (excluding comments) |
| Characters per line | **100 max** |

---

## Article III: The CI Gauntlet

Every submission must survive three trials:

1. **Test** — `cargo build` + `cargo test` must succeed
2. **Lint** — `cargo fmt --check` + `cargo clippy -D warnings` must pass
3. **Dictator** — `dictator lint .` must report compliance

All three jobs must display green status. Yellow is not green. Red is treason.

---

## Article IV: The Pull Request Protocol

### Draft Until Green

All pull requests **MUST** be opened as drafts.

A PR may only be marked "Ready for Review" when:
- All CI checks are green
- The branch is rebased on `master`
- Commit history is clean

### No Vibes

PRs are not merged on:
- "It works on my machine"
- "The tests are flaky"
- "I'll fix it in the next PR"
- Good intentions

PRs are merged on:
- Green CI
- Code review approval
- Compliance with all decrees

### Commit Messages

Follow Conventional Commits or face refactoring camp:

```
feat: add new decree for YAML ordering
fix: correct line ending detection on Windows
refactor: simplify violation reporting
docs: update submission guidelines
chore: bump dependencies
```

---

## Article V: Local Compliance Check

Before pushing, citizens should verify compliance locally:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace -- -D warnings

# Run tests
cargo test --workspace

# Run the Dictator
cargo run -p dictator -- lint .
```

Or use the auto-fix decree:

```bash
cargo run -p dictator -- fix .
```

---

## Article VI: Consequences

Non-compliant submissions are:

1. Automatically rejected by CI
2. Not reviewed by maintainers
3. Closed after 14 days of inactivity

There is no appeals process. Fix the violations and resubmit.

---

## Article VII: The Dictator's Promise

The Dictator enforces these rules equally. No contributor is above the law.
No legacy code is exempt. No "quick fix" bypasses the gauntlet.

Compliance is not optional. Compliance is the baseline.

---

*This document is automatically enforced. Your excuses are not.*
