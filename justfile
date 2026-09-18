# Dictator justfile - The Dictator decrees efficient workflows

# List available commands
default:
    @just --list

# === Build Commands ===

# Build all crates in debug mode
build:
    cargo build --workspace

# Build all crates in release mode
build-release:
    cargo build --workspace --release

# Build the main CLI only
build-cli:
    cargo build -p dictator --release

# === Test Commands ===

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run clippy checks
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# === Release Commands ===

# Linked-version crates (the release-please group) - drives `versions` and `bump`
CRATES := "dictator-decree-abi dictator-core dictator-supreme dictator-frontmatter dictator-ruby dictator-typescript dictator-golang dictator-rust dictator-python dictator"

# Everything that goes to crates.io. Decree crates version independently, but
# `dictator` depends on dictator-freebsd, so it must ship in the same pass.
# dictator-coreboot stays unpublished; dictator-kjr is `publish = false`.
PUBLISH_CRATES := CRATES + " dictator-freebsd"

# Dry-run the full publish set - packages and verifies, uploads nothing
publish-dry:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo publish --dry-run $(printf -- '-p %s ' {{PUBLISH_CRATES}})

# Publish all crates to crates.io (requires login)
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    # cargo resolves publish order and waits on the index itself
    cargo publish $(printf -- '-p %s ' {{PUBLISH_CRATES}})
    echo "All crates published. The Dictator is pleased."

# Publish a specific crate
publish-crate crate:
    cargo publish -p {{crate}}

# === Version Management ===

# Show the workspace version and flag any crate that stopped inheriting it
versions:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "workspace: $(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
    drift=$(grep -l '^version = "' crates/*/Cargo.toml | grep -v dictator-kjr || true)
    if [ -n "$drift" ]; then
        echo "drift - these pin their own version instead of inheriting:" >&2
        echo "$drift" >&2
        exit 1
    fi

# Set the workspace version by hand. release-please normally owns this;
# `cargo set-version` cannot write an inherited version (cargo-edit#752).
set-version version:
    #!/usr/bin/env bash
    set -euo pipefail
    python3 - <<'PY'
    import re, pathlib
    p = pathlib.Path("Cargo.toml")
    t = p.read_text()
    t = re.sub(r'^version = "[^"]+"', 'version = "{{version}}"', t, count=1, flags=re.M)
    t = re.sub(r'^(dictator-[a-z-]+ = \{ version = )"[^"]+"', r'\g<1>"{{version}}"', t, flags=re.M)
    p.write_text(t)
    PY
    cargo check --workspace --quiet
    echo "Workspace pinned at {{version}}. The Dictator decrees uniformity."

# === Quality Assurance ===

# Run all checks (format, lint, test)
check: fmt-check lint test
    @echo "All checks passed"

# Pre-release checklist
pre-release: check publish-dry
    @echo "Ready for release"

# === Clean ===

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# === MCP Server ===

# Run dictator as MCP server
mcp:
    cargo run -p dictator -- mcp

# Run dictator lint on current directory
dictate *args:
    cargo run -p dictator -- lint {{args}}
