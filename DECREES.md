# Building Decrees

Decrees are structural enforcement rules for Dictator. This guide shows how to build your own.

**Don't know Rust?** No problem. A WASM decree is a WebAssembly component exporting the WIT interface in `crates/dictator-decree-abi/wit/decree.wit`, so any language with component-model tooling works (Go, Python, C, JavaScript, ...). Dictator is the runtime; it doesn't care how you built the decree.

**Vibe-coding works too.** Ask Claude or another LLM to implement the `Decree` trait for your use case. The interface is minimal: `name()`, `lint()`, and `metadata()`. Show it the trait and the WIT below and describe what you want to lint.

## Quick Start

```bash
cargo new --lib my-decree
cd my-decree
```

`Cargo.toml`:
```toml
[package]
name = "my-decree"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
dictator-decree-abi = "0.25"   # the Decree trait and ABI_VERSION; serde-only, builds for wasm
wit-bindgen = "0.62"           # WASM component bindings

[profile.release]
opt-level = "z"
lto = true
strip = true
```

The decree ABI moves with Dictator releases: build against the `dictator-decree-abi` matching the Dictator you run.

## Implementing the Decree Trait

```rust
use dictator_decree_abi::{Capability, Decree, DecreeMetadata, Diagnostic, Diagnostics, Span};

#[derive(Default)]
pub struct MyDecree;

impl Decree for MyDecree {
    fn name(&self) -> &str {
        "my-decree"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();

        // Your enforcement logic here
        if let Some(start) = source.find("forbidden") {
            diags.push(Diagnostic {
                rule: self.rule("no-forbidden"), // "my-decree/no-forbidden"
                message: "forbidden".to_string(),
                enforced: false,                 // you must comply (see below)
                span: Span::new(start, start + "forbidden".len()),
            });
        }

        diags
    }

    fn metadata(&self) -> DecreeMetadata {
        DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "My custom decree for enforcement".to_string(),
            persona: "The Censor".to_string(),   // who speaks in diagnostics
            dectauthors: Some("Your Name <you@example.com>".to_string()),
            supported_extensions: vec!["txt".to_string()],
            supported_filenames: vec![],         // exact filenames to match
            skip_filenames: vec![],              // files to own but not lint
            file_scope_rules: vec![],            // bare names of whole-file rules
            capabilities: vec![Capability::Lint],
        }
    }
}
```

## Shipping a Decree

A decree ships as a **WASM component** (sandboxed, portable; the recommended form) or as a **native dynamic library** (`.dylib`/`.so`/`.dll`, full trust). The file extension decides: `.wasm` loads as a component, anything else as a native library.

### WASM component

Bridge the trait to the WIT exports with wit-bindgen. The WIT ships inside the `dictator-decree-abi` crate; copy it next to your crate (`wit/decree.wit`):

```rust
wit_bindgen::generate!({
    path: "wit/decree.wit",
    world: "decree",
});

use exports::dictator::decree::lints::{self as wit, Guest};

struct Component;

impl Guest for Component {
    fn name() -> String {
        MyDecree.name().to_string()
    }

    fn lint(path: String, source: String) -> Vec<wit::Diagnostic> {
        MyDecree
            .lint(&path, &source)
            .into_iter()
            .map(|d| wit::Diagnostic {
                rule: d.rule,
                message: d.message,
                // The host maps Info to enforced = true, Error and Warning to false.
                severity: if d.enforced { wit::Severity::Info } else { wit::Severity::Error },
                span: wit::Span {
                    start: u32::try_from(d.span.start).unwrap_or(u32::MAX),
                    end: u32::try_from(d.span.end).unwrap_or(u32::MAX),
                },
            })
            .collect()
    }

    fn metadata() -> wit::DecreeMetadata {
        let m = MyDecree.metadata();
        wit::DecreeMetadata {
            abi_version: m.abi_version,
            decree_version: m.decree_version,
            description: m.description,
            persona: m.persona,
            dectauthors: m.dectauthors,
            supported_extensions: m.supported_extensions,
            supported_filenames: m.supported_filenames,
            skip_filenames: m.skip_filenames,
            file_scope_rules: m.file_scope_rules,
            capabilities: vec![wit::Capability::Lint],
        }
    }
}

export!(Component);
```

Build a component. Dictator loads decrees with wasmtime 49 as components (`Component::from_file`); a plain core module does not load.

```bash
# A) wasm32-wasip2: rustc emits a component directly
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/my_decree.wasm my-decree.component.wasm

# B) wasm32-wasip1 + the WASI adapter from the matching wasmtime release
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
curl -LO https://github.com/bytecodealliance/wasmtime/releases/download/v49.0.2/wasi_snapshot_preview1.reactor.wasm
wasm-tools component new target/wasm32-wasip1/release/my_decree.wasm \
  --adapt wasi_snapshot_preview1=wasi_snapshot_preview1.reactor.wasm \
  -o my-decree.component.wasm

# Check it exports the decree interface
wasm-tools validate my-decree.component.wasm
wasm-tools component wit my-decree.component.wasm   # expect: export dictator:decree/lints@0.2.0
```

### Native library

Export the factory under the name in `dictator_decree_abi::DECREE_FACTORY_EXPORT`:

```rust
use dictator_decree_abi::BoxDecree;

#[unsafe(no_mangle)]
pub fn dictator_create_decree() -> BoxDecree {
    Box::new(MyDecree)
}
```

The factory is a Rust-ABI `fn() -> BoxDecree`, so build the library with the same Rust toolchain and `dictator-decree-abi` version as the Dictator binary that loads it. A native decree runs unsandboxed inside Dictator: load only libraries you trust.

## Decree Versioning & Metadata

Every decree provides metadata for ABI compatibility checking, so a decree built against another Dictator fails to load with a clear error naming both versions.

**ABI versioning rules (pre-1.0):**
- A decree's `abi_version` must match Dictator's ABI exactly in major.minor.
- A decree built with ABI 0.2.x loads in a Dictator with ABI 0.2.y; one built with ABI 0.1.x does not.
- Post-1.0: the major must match and the decree's minor must not exceed the host's.

**Metadata fields:**
- `abi_version`: always `dictator_decree_abi::ABI_VERSION`
- `decree_version`: your decree's version from `Cargo.toml`
- `description`: human-readable description
- `persona`: the named persona that speaks for the decree in diagnostics (Ferris, Matz, Beastie, ...)
- `dectauthors`: who authored the decree (optional)
- `supported_extensions`: file extensions this decree handles (e.g. `["rb", "rake"]`)
- `supported_filenames`: exact filenames to match (e.g. `["Gemfile", "Rakefile"]`)
- `skip_filenames`: files to own but NOT lint (lock files, generated files)
- `file_scope_rules`: bare rule names (`"file-too-long"`, no `my-decree/` prefix) whose span covers the whole file or sits at 0/EOF; `lint --diff` reports them whenever the file is touched
- `capabilities`: declared features (`[Capability::Lint]`)

**Example:**
```rust
fn metadata(&self) -> DecreeMetadata {
    DecreeMetadata {
        abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
        decree_version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Ruby hygiene and structure rules".to_string(),
        persona: "Matz".to_string(),
        dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
        supported_extensions: vec!["rb".to_string(), "rake".to_string()],
        supported_filenames: vec!["Gemfile".to_string(), "Rakefile".to_string()],
        skip_filenames: vec!["Gemfile.lock".to_string()],
        file_scope_rules: vec!["file-too-long".to_string()],
        capabilities: vec![Capability::Lint],
    }
}
```

## File Matching

Decrees match files using three mechanisms:

1. **Extensions** (`supported_extensions`): match by file extension (e.g. `.rb`, `.go`)
2. **Filenames** (`supported_filenames`): match exact filenames (e.g. `Gemfile`, `go.mod`)
3. **Skip** (`skip_filenames`): claim ownership but don't lint (lock files, generated)

**Matching priority:**
1. If `skip_filenames` matches → decree owns the file, returns empty diagnostics
2. If `supported_filenames` matches → lint the file
3. If `supported_extensions` matches → lint the file
4. If both lists are empty → universal decree (matches all files)

**Why skip files?**
Lock files (`Gemfile.lock`, `package-lock.json`, `Cargo.lock`) are auto-generated.
Without `skip_filenames`, the universal `supreme` decree would complain about line length, trailing whitespace, etc. By claiming ownership with skip, the language decree prevents this.

## The `enforced` Field

| Value | Meaning | Display |
|-------|---------|---------|
| `true` | Dictator enforces it: `dictate` / `lint --fix` rewrites the file | 🔧 |
| `false` | User must comply | ❌ |

Dictator rewrites only rules whose name contains a fix it knows: `trailing-whitespace`, `tab-character`, `missing-final-newline`, `mixed-line-endings` / `wrong-line-ending`, `blank-line-whitespace`. A decree cannot ship its own fixer, so a custom rule marked `enforced: true` is shown as fixed while nothing changes the file. Use `false` for everything else.

Use the `self.rule("name")` helper: it builds `"decree/name"` without duplicating the decree name.

## Loading a Decree

Decrees load only from an explicit path. Add to `.dictate.toml`:

```toml
[decree.my-decree]
enabled = true                                   # default true when `path` is set
path = "decrees/my-decree.component.wasm"        # .wasm = component; anything else = native library

# Per-rule ignores, applied by the host after the decree runs.
# Exact filenames and extensions; no globs.
[decree.my-decree.ignore.no-forbidden]
filenames = ["CHANGELOG.txt"]
extensions = ["log"]
```

Or ad hoc: `dictator lint --plugin my-decree.component.wasm src/` (`--plugin` also works on `watch`).

- Built-in decree names (`supreme`, `ruby`, `typescript`, `golang`, `rust`, `python`, `frontmatter`, `freebsd`) never load from `path`. Give a custom decree its own name.
- Decrees receive no settings: `lint` gets only `path` and `source`. Keys such as `max_lines` under `[decree.<name>]` configure the built-in decrees. Hard-code thresholds in your decree, or ship variants.
- An ABI mismatch fails the run with both versions in the error. Rebuild against the matching `dictator-decree-abi`.
- `dictator census` shows which decrees loaded.

## Example: decree.kjr

See `crates/dictator-kjr/` for a complete enterprise-grade decree with:
- 16 productivity-focused enforcement rules
- A native `Decree` implementation wrapped in a WASM component `Guest` (the pattern above)
- Comprehensive test coverage
- GPL licensed (Glorious People Licence)

It sits outside the workspace (`exclude` in the root `Cargo.toml`); build it from its own directory with the commands in [WASM component](#wasm-component).

## Tips

1. **Keep it fast** - Decrees run on every file. Avoid heavy parsing.
2. **Use Span correctly** - `Span::new(start, end)` takes byte offsets into the source, half-open.
3. **Use `self.rule()`** - DRY helper: `self.rule("name")` → `"decree/name"`.
4. **Concise messages** - LLMs read these. Fewer tokens = better.
5. **Test natively first** - Unit-test the `Decree` impl with `cargo test`, then build the component.

## Built-in Decrees

Built-in decrees are compiled into the Dictator binary. A language decree loads when the tree contains its files and `.dictate.toml` has a `[decree.<name>]` section without `enabled = false` (`dictator occupy` writes a starting config).

| Decree | Persona | Crate | Extensions | Filenames | Skip |
|--------|---------|-------|------------|-----------|------|
| supreme | The Supreme Leader | `dictator-supreme` | ALL | - | - |
| ruby | Matz | `dictator-ruby` | `rb`, `rake`, `gemspec` | `Gemfile`, `Rakefile`, `Guardfile`, etc. | `Gemfile.lock` |
| typescript | Anders | `dictator-typescript` | `ts`, `tsx`, `js`, `jsx`, `mjs`, `cjs` | `package.json`, `tsconfig.json`, etc. | `package-lock.json`, `yarn.lock`, etc. |
| golang | Gopher | `dictator-golang` | `go` | `go.mod`, `go.work` | `go.sum` |
| rust | Ferris | `dictator-rust` | `rs` | `Cargo.toml`, `build.rs`, etc. | `Cargo.lock` |
| python | Monty | `dictator-python` | `py`, `pyi`, `pyw` | `pyproject.toml`, `setup.py`, `Pipfile`, etc. | `Pipfile.lock`, `poetry.lock`, `uv.lock` |
| frontmatter | The Registrar | `dictator-frontmatter` | `md`, `mdx`, `astro` | - | - |
| freebsd | Beastie | `dictator-freebsd` | `c`, `h`, `1`-`9` | - | - |

These serve as implementation references. Check their source for patterns.
