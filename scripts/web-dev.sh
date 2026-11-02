#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
command -v trunk >/dev/null || { echo "Run ./scripts/setup.sh first." >&2; exit 1; }
cd client
exec trunk serve "$@"
