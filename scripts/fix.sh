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
chmod 744 ./scripts/**/*.sh

info Fixing errors in Bash script files
# The command fails if there are unfixable errors
set +e
shellcheck -xf diff ./scripts/**/*.sh | patch -p1
set -e

info Formatting Bash script files
shfmt -i 2 -ci --write ./scripts/**/*.sh

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
addlicense -s -l "MPL-2.0" ./**/*

info Removing unnecesary Rust dependencies
cargo machete --fix
