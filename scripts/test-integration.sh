#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
# Requires db-start.sh and db-publish.sh. No publication or data reset here.
SPACETIME_TEST_SERVER="ws://127.0.0.1:3001" \
SPACETIME_TEST_DATABASE="$DATABASE" \
cargo test --locked -p project-tactics-client --no-default-features --test presence -- --ignored --nocapture
