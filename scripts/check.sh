#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != "--native" ) ]]; then
  echo "Usage: $0 [--native]" >&2
  exit 2
fi
cargo fmt --all --check
cargo check --locked -p project-tactics-server --target wasm32-unknown-unknown
cargo check --locked -p project-tactics-client --target wasm32-unknown-unknown
cargo check --locked -p tui-starter
# Native is an optional secondary target with additional OS dependencies.
if [[ "${1:-}" == "--native" ]]; then
  cargo test --locked -p project-tactics-client --no-default-features --lib
fi
