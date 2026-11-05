# Project Tactics

A web-first **Bevy 0.20.1 + SpacetimeDB 2.14.0** foundation, following
`../spacetime-bevy/`. The web client has no gameplay: its connected-client probe
shows a live list of connections, with yours marked **(you)**. The workspace also
contains an independent offline **Genome Lab** legacy Ratatui prototype. Its
revised target is a Rust web-first lab with seeded genome generation, direct
genome editing, and rich creature inspection; the web framework is undecided.

## Setup

Requirements: Rust **1.98+**, SpacetimeDB CLI **2.14.0**, a WebGL2 browser.
`setup.sh` adds the WASM target, rustfmt and rust-analyzer, and installs Trunk
**0.22.1** into ignored `.tools/` if it is absent. Install the CLI separately using its official installer.

```sh
./scripts/setup.sh
```

For headless native tests on Ubuntu/Debian, install `pkg-config libssl-dev`.
Optional native windowed runs also need `libx11-dev libxkbcommon-dev`.
These are not required for the browser-only check/build. First builds can take
several minutes; Cargo limits parallelism to two jobs.

## Local development

Run from the repo root, in separate terminals:

```sh
# 1. Persistent database, loopback only
./scripts/db-start.sh

# 2. Initial publication (repeat after server changes), then regenerate bindings
./scripts/db-publish.sh
./scripts/bindings.sh

# 3. Browser dev server with rebuilds
./scripts/web-dev.sh
```

Open **http://127.0.0.1:8082**. A second tab adds another connection to both lists;
closing it removes that row after the server detects the disconnect. Reload or
focus the canvas and press **R** to reconnect. Errors appear in the page and Bevy
overlay. The DOM mirrors the canvas text for accessibility and browser testing;
the DOM list remains readable/scrollable even when many rows exceed the canvas.

This project uses **3001 / 8082** and database **project-tactics-dev**, deliberately
separate from the reference project's 3000 / 8080. Data and the local publisher's
credentials are stored in ignored `.local/`. Keep `.local/cli.toml` to retain
publication ownership. Publication refuses data deletion and does not inherit
your global cloud default. Schema-breaking changes need explicit review.

## Genome Lab

The retired Genome Lab TUI, domain engine, and sample experiments have been
removed. Its [web-first specification](.scratch/creature-mmo/specs/genome-lab.md)
remains the target for a future implementation and is not yet implemented.
Historical issues 12 and 13 remain as records of the former design and work.
The ignored `.local/genome-lab.json` store, if present, is deliberately left
untouched and is not used by the project.

## Ratatui starter

`tools/tui-starter` is a small, independent Rust terminal UI scaffold using the
workspace-pinned Ratatui and Crossterm versions. It demonstrates terminal setup,
a minimal event/render loop, and terminal restoration. Use it as a starting point
for future unrelated TUIs; it contains no Genome Lab functionality.

```sh
cargo run --locked -p tui-starter
```

Press `q` to quit. The starter's test uses Ratatui's in-memory test backend.

## Verification

```sh
./scripts/check.sh                # formatting + both WASM targets + starter check
./scripts/test.sh                 # headless client + starter tests
./scripts/test-integration.sh     # real SDK/server; requires running, published DB
./scripts/verify-bindings.sh      # regenerate into temporary directory and diff
./scripts/web-build.sh            # release web bundle
```

`check.sh --native` also runs client unit tests. Client tests use
`--no-default-features` to exercise the network library headlessly without
compiling the renderer. Both scripts also cover the lab packages. The integration test opens connections
sharing an identity, verifies replication to both, removes one, and reconnects.
It tolerates unrelated connected browsers and removes its own sessions on exit.
It does not prove rendering. Follow [browser QA](docs/browser-qa.md) for that.

CI runs WASM checks, unit and live integration tests, binding drift checks, and a
release web build. The bundle is uploaded as a workflow artifact.

## Distribution

```sh
SPACETIME_SERVER=wss://your-database-host \
SPACETIME_DATABASE=your-database-name ./scripts/web-build.sh
```

Deploy **client/dist/** to any static HTTPS host; serve `.wasm` files with MIME
type `application/wasm`, preserving hashed filenames. For subpath hosting, pass
`--public-url /your-path/` to the build script. The database is a separate
deployment: no production database publication script is provided.

Browser endpoint overrides are also supported for diagnosis:
`?server=ws%3A%2F%2F127.0.0.1%3A3001&database=project-tactics-dev`.
These choose the endpoint receiving your connection; never put credentials in
URLs. Native runs accept runtime `SPACETIME_SERVER` / `SPACETIME_DATABASE`.
Production must use a publicly reachable WSS endpoint, not localhost.

See [architecture](docs/architecture.md) for boundaries, extension points and scope.
