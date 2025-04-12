#!/usr/bin/env bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail
shopt -s globstar

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit

source ./scripts/modules/_logging.sh

info Setting Bash script permissions
for script in ./scripts/**/*.sh; do
  if [ "$(stat -c "%a" "$script")" != "744" ]; then
    echo "The script '$script' doesn't have the correct permission"
    exit 1
  fi
done

info Checking errors in Bash script files
shellcheck ./scripts/**/*.sh

info Checking format Bash script files
shfmt -i 2 -ci --diff ./scripts/**/*.sh

info Formatting Nix files
nix fmt flake.nix -- --check

info Formatting TOML files
taplo lint

info Formatting JSON files
prettier --check ./**/*.json

info Checking fixes in Rust code
cargo check

info Checking formatting in Rust code
cargo fmt --check

info Checking license headers
addlicense -s -l "MPL-2.0" -check ./**/*

info Checking unnecesary Rust dependencies
cargo machete --fix
