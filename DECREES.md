# Building Decrees

Decrees are structural enforcement rules for Dictator. This guide shows how to build your own.

## Quick Start

```bash
# Create a new decree crate
cargo new --lib my-decree
cd my-decree
```

Add to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
dictator-decree-abi = { git = "https://github.com/seuros/dictator" }

[profile.release]
opt-level = "z"
lto = true
```

## Implementing the Decree Trait

```rust
use dictator_decree_abi::{BoxDecree, Capability, DecreeMetadata, Diagnostic, Diagnostics, Decree, Span};

pub struct MyDecree;

impl Decree for MyDecree {
    fn name(&self) -> &str {
        "my-decree"
    }

    fn lint(&self, path: &str, source: &str) -> Diagnostics {
        let mut diags = Diagnostics::new();

        // Your enforcement logic here
        if source.contains("forbidden") {
            diags.push(Diagnostic {
                rule: self.rule("no-forbidden"), // DRY: "my-decree/no-forbidden"
                message: "forbidden".to_string(),
                enforced: false, // User must fix
                span: Span::new(0, 9),
            });
        }

        diags
    }

    fn metadata(&self) -> DecreeMetadata {
        DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "My custom decree for enforcement".to_string(),
            dectauthors: Some("Your Name <you@example.com>".to_string()),
            supported_extensions: vec!["txt".to_string()],
            capabilities: vec![Capability::Lint],
        }
    }
}

pub fn init_decree() -> BoxDecree {
    Box::new(MyDecree)
}
```

## Decree Versioning & Metadata

Every decree must provide metadata for ABI compatibility checking. This ensures decrees built against different Dictator versions fail cleanly with clear errors.

**ABI Versioning Rules (Pre-1.0):**
- Decrees must have `abi_version` matching Dictator's ABI version exactly (major.minor)
- Example: Decree built with `dictator-decree-abi 0.1.0` works with Dictator ABI 0.1.x
- Decree built with `dictator-decree-abi 0.2.0` will NOT load in Dictator with ABI 0.1.0

**Metadata Fields:**
- `abi_version`: Use `dictator_decree_abi::ABI_VERSION` constant (always correct)
- `decree_version`: Your decree's version from `Cargo.toml`
- `description`: Human-readable description
- `author`: Who authored the decree (optional)
- `supported_extensions`: File extensions this decree handles (e.g., `["rb", "rake"]`)
- `capabilities`: Declare supported features (usually just `[Capability::Lint]` for now)

**Example:**
```rust
fn metadata(&self) -> DecreeMetadata {
    DecreeMetadata {
        abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
        decree_version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Ruby hygiene and structure rules".to_string(),
        dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
        supported_extensions: vec!["rb".to_string(), "rake".to_string()],
        capabilities: vec![Capability::Lint],
    }
}
```

## The `enforced` Field

| Value | Meaning | Display |
|-------|---------|---------|
| `true` | Dictator auto-fixed | 🔧 |
| `false` | User must comply | ❌ |

Use `self.rule("name")` helper - DRY, no string duplication.

## Building as WASM

Decrees can be distributed as WASM components for sandboxed execution.

```bash
# 1. Add WASM target
rustup target add wasm32-wasip1

# 2. Build
cargo build --target wasm32-wasip1 --release

# 3. Download WASI adapter (one-time)
curl -LO https://github.com/bytecodealliance/wasmtime/releases/download/v39.0.1/wasi_snapshot_preview1.reactor.wasm

# 4. Convert to component
wasm-tools component new \
  target/wasm32-wasip1/release/my_decree.wasm \
  --adapt wasi_snapshot_preview1=wasi_snapshot_preview1.reactor.wasm \
  -o my-decree.component.wasm
```

## Loading via Config

Add to `.dictate.toml`:

```toml
[decree.my-decree]
enabled = true
path = "path/to/my-decree.component.wasm"
```

## Example: decree.kjr

See `crates/dictator-kjr-decree/` for a complete enterprise-grade decree with:
- 16 productivity-focused enforcement rules
- Comprehensive test coverage
- WASM build configuration
- GPL licensed (Glorious People Licence)

```bash
# Build KJR as reference
cd crates/dictator-kjr-decree
cargo build --target wasm32-wasip1 --release
```

## Tips

1. **Keep it fast** - Decrees run on every file. Avoid heavy parsing.
2. **Use Span correctly** - `Span::new(start, end)` for byte offsets in source.
3. **Use `self.rule()`** - DRY helper: `self.rule("name")` → `"decree/name"`.
4. **Concise messages** - LLMs read these. Less tokens = better.
5. **Test locally** - Build as native first, then WASM.

## Native Decrees

Native decrees are compiled into the Dictator binary:

| Decree | Crate | Files |
|--------|-------|-------|
| supreme | `dictator-supreme` | ALL |
| ruby | `dictator-ruby` | `*.rb` |
| typescript | `dictator-typescript` | `*.ts`, `*.js` |
| golang | `dictator-golang` | `*.go` |
| rust | `dictator-rust` | `*.rs` |
| python | `dictator-python` | `*.py` |
| frontmatter | `dictator-frontmatter` | `*.md`, `*.mdx` |

These serve as implementation references. Check their source for patterns.
