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

# Crates in dependency order for publishing
CRATES := "dictator-decree-abi dictator-core dictator-supreme dictator-frontmatter dictator-ruby dictator-typescript dictator-golang dictator-rust dictator-python dictator"

# Dry-run publish base crate (full chain requires sequential publishing)
publish-dry:
    cargo publish -p dictator-decree-abi --dry-run

# Publish all crates to crates.io (requires login)
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{CRATES}}; do
        echo "Publishing $crate..."
        cargo publish -p "$crate"
        echo "$crate published"
        # Sleep to let crates.io index
        if [ "$crate" != "dictator" ]; then
            echo "Waiting for crates.io to index..."
            sleep 30
        fi
    done
    echo "All crates published. The Dictator is pleased."

# Publish a specific crate
publish-crate crate:
    cargo publish -p {{crate}}

# === Version Management ===

# Show current versions of all crates
versions:
    #!/usr/bin/env bash
    for crate in {{CRATES}}; do
        version=$(grep -m1 '^version' "crates/$crate/Cargo.toml" | cut -d'"' -f2)
        echo "$crate: $version"
    done

# Bump version (patch/minor/major) - requires cargo-edit
bump level:
    #!/usr/bin/env bash
    set -euo pipefail
    for crate in {{CRATES}}; do
        echo "Bumping $crate..."
        cargo set-version -p "$crate" --bump {{level}}
    done
    echo "Versions bumped. Don't forget to update inter-crate dependencies!"

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
