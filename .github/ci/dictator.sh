#!/usr/bin/env bash
set -euo pipefail

# Lint with the code under test, not the last published Dictator.
mbx run --locked -p dictator -- -c .dictate.ci.toml lint \
  crates/{dictator,dictator-core,dictator-supreme,dictator-ruby,dictator-typescript,dictator-golang,dictator-rust,dictator-python,dictator-frontmatter}/src
