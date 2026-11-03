#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/.tools/bin:$PATH"
# Native SDK tests need OpenSSL headers. In minimal WSL/container images the
# runtime libraries may be installed without the development package. Use the
# repo-local extracted headers and system runtime when that is the case.
OPENSSL_MULTIARCH="$(gcc -print-multiarch 2>/dev/null || true)"
OPENSSL_LOCAL="$ROOT/.tools/openssl/usr"
if [[ -n "$OPENSSL_MULTIARCH" && -d "$OPENSSL_LOCAL/include/openssl" && ! -f "/usr/include/openssl/ssl.h" ]]; then
  OPENSSL_LOCAL_LIB="$OPENSSL_LOCAL/lib/$OPENSSL_MULTIARCH"
  OPENSSL_RUNTIME_LIB="/usr/lib/$OPENSSL_MULTIARCH"
  for library in ssl crypto; do
    runtime="$OPENSSL_RUNTIME_LIB/lib$library.so"
    [[ -e "$runtime" ]] || runtime="$(find "$OPENSSL_RUNTIME_LIB" -maxdepth 1 -name "lib$library.so.*" -print -quit)"
    if [[ -n "$runtime" && ! -e "$OPENSSL_LOCAL_LIB/lib$library.so" ]]; then
      ln -sfn "$runtime" "$OPENSSL_LOCAL_LIB/lib$library.so"
    fi
  done
  export OPENSSL_INCLUDE_DIR="$OPENSSL_LOCAL/include"
  export OPENSSL_LIB_DIR="$OPENSSL_LOCAL_LIB"
fi
LOCAL_SERVER="http://127.0.0.1:3001"
DATABASE="project-tactics-dev"
require_cli() {
  command -v spacetime >/dev/null || { echo "Install SpacetimeDB CLI 2.14.0 first." >&2; exit 1; }
  spacetime --version | grep -q 'tool version 2.14.0;' || {
    echo "Use SpacetimeDB CLI 2.14.0 to match the pinned SDKs." >&2; exit 1;
  }
}
