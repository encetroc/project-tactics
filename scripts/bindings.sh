#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
require_cli
exec spacetime generate --no-config --lang rust --module-path server \
  --out-dir client/src/module_bindings --yes
