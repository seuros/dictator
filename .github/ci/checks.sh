#!/usr/bin/env bash
set -euo pipefail

# Isolate core so other workspace members cannot enable its WASM loader.
mbx check --locked -p dictator-core --all-targets --no-default-features
mbx test --locked -p dictator-core --no-default-features
