#!/usr/bin/env bash
# One-time dev setup: installs the tracked pre-commit hook for this clone.
# Usage: ./scripts/setup-hooks.sh
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
hooks_dir="$repo_root/.git/hooks"
src="$repo_root/scripts/git-hooks/pre-commit"

mkdir -p "$hooks_dir"
cp "$src" "$hooks_dir/pre-commit"
chmod +x "$hooks_dir/pre-commit"

echo "Installed pre-commit hook -> $hooks_dir/pre-commit"
