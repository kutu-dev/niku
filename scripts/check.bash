#!/usr/bin/env bash
set -euo pipefail
shopt -s globstar

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit

source ./scripts/modules/_logging.bash

info Setting Bash script permissions
chmod u+x ./scripts/**/*.bash
for script in ./scripts/**/*.bash; do
  if [ "$(stat -c "%a" scripts/check.bash)" != "744" ]; then
    echo "The script '$script' doesn't have the correct permission"
    exit 1
  fi
done

info Fixing errors in Bash script files
shellcheck ./scripts/**/*.bash

info Formatting Bash script files
shfmt -i 2 -ci --diff ./scripts/**/*.bash

info Formatting Nix files
nix fmt flake.nix -- --check

info Formatting TOML files
taplo lint

info Formatting JSON files
prettier --check ./**/*.json

info Checking fixes in Rust code
cargo fix --allow-dirty --check

info Checking formatting in Rust code
cargo +nightly fmt --allow-dirty --check

info Checking license headers
# TODO:
#addlicense -s -l "MPL-2.0" -c 'Jorge "Kutu" Dobón Blanco' -check ./**/*

info Checking unnecesary Rust dependencies
cargo machete --fix
