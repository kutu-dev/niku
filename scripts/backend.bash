#!/usr/bin/env bash
set -euo pipefail
shopt -s globstar

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit

source ./scripts/modules/_logging.bash

niku_backend_log_level="debug"

if [ $1 == "trace" ]; then
  info Enabling tracing log level
  niku_backend_log_level="trace"
fi

APP_NIKU_BACKEND_PORT="8080" RUST_LOG="niku_backend=$niku_backend_log_level,info" cargo run --bin niku_backend
