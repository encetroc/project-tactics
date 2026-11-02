#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
cd client
exec trunk build --release "$@"
