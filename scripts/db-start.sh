#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
require_cli
mkdir -p "$ROOT/.local"
exec spacetime start --listen-addr 127.0.0.1:3001 --data-dir "$ROOT/.local/spacetimedb" --non-interactive
