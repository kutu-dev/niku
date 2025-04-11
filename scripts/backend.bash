#!/usr/bin/env bash
set -euo pipefail
shopt -s globstar

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit

APP_NIKU_BACKEND_PORT="8080" RUST_LOG="niku_backend=debug,info" cargo run --bin niku_backend
