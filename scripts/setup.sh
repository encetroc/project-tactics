#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
require_cli
rustup target add wasm32-unknown-unknown
rustup component add rustfmt rust-analyzer
if ! command -v trunk >/dev/null; then
  cargo install trunk --version 0.22.1 --locked --root "$ROOT/.tools"
fi
trunk --version | grep -qx 'trunk 0.22.1' || {
  echo "Use Trunk 0.22.1 to match the reference toolchain." >&2; exit 1;
}
cargo fetch --locked
echo "Ready. See README.md for the three-terminal development workflow."
