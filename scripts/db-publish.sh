#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
require_cli
# Explicit loopback URL and no config: never inherit a maincloud default.
# Refuse data deletion and leave schema-breaking changes for human review.
mkdir -p "$ROOT/.local"
umask 077
spacetime --config-path "$ROOT/.local/cli.toml" publish --no-config \
  --server "$LOCAL_SERVER" --module-path server \
  --delete-data=never --yes=skip-login "$DATABASE"
