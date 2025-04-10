#!/usr/bin/env bash
set -euo pipefail
shopt -s globstar

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit

source ./scripts/modules/_logging.bash

ls ./scripts/**/*.bash

info Setting Bash script permissions
chmod 744 ./scripts/**/*.bash

info Fixing errors in Bash script files
shellcheck -xf diff ./scripts/**/*.bash | git apply --allow-empty

info Formatting Bash script files
shfmt -i 2 -ci --write ./scripts/**/*.bash

info Formatting Nix files
nix fmt flake.nix

info Formatting TOML files
taplo format

info Formatting JSON files
prettier --write ./**/*.json

info Fixing Rust code
cargo fix --allow-dirty --allow-staged

info Formatting Rust code
cargo fmt

info Adding license headers
addlicense -s -l MPL ./*
