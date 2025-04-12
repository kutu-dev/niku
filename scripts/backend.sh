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

niku_backend_log_level="debug"

if [ ${1-__placeholder} != "__placeholder" ]; then
    info "Enabling '$1' log level"
    niku_backend_log_level="$1"
fi

APP_NIKU_BACKEND_PORT="8080" RUST_LOG="niku_backend=$niku_backend_log_level,info" cargo run --bin niku_backend
