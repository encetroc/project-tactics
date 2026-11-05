#!/usr/bin/env bash
source "$(dirname -- "$0")/common.sh"
cargo test --locked -p project-tactics-client --no-default-features --lib
cargo test --locked -p tui-starter
