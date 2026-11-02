#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
require_cli
temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT
spacetime generate --no-config --lang rust --module-path server --out-dir "$temp" --yes
# Keep generated formatting aligned with cargo fmt.
find "$temp" -name '*.rs' -exec rustfmt --edition 2024 {} +
diff -ru client/src/module_bindings "$temp"
