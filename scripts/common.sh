#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/.tools/bin:$PATH"
LOCAL_SERVER="http://127.0.0.1:3001"
DATABASE="project-tactics-dev"
require_cli() {
  command -v spacetime >/dev/null || { echo "Install SpacetimeDB CLI 2.14.0 first." >&2; exit 1; }
  spacetime --version | grep -q 'tool version 2.14.0;' || {
    echo "Use SpacetimeDB CLI 2.14.0 to match the pinned SDKs." >&2; exit 1;
  }
}
