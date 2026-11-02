# Architecture

## Boundaries

- `server/src/lib.rs`: authoritative public presence table and connection lifecycle
  reducers. One row per **ConnectionId**, including the caller's **Identity**.
  Connection IDs distinguish tabs even when identities are shared.
- `client/src/module_bindings/`: committed CLI-generated protocol adapter. Change
  the schema first, regenerate bindings second; never hand-edit these files.
- `client/src/network.rs`: the only handwritten SDK integration. It owns the
  connection, validates endpoint configuration, pumps updates once per frame, and
  exposes a sorted, read-only-in-practice `ConnectionSnapshot` to presentation.
- `client/src/main.rs`: Bevy rendering plus accessible DOM text mirror. Reads the
  snapshot; has no authority over server presence.
- `client/tests/presence.rs`: real protocol/lifecycle test, separate from offline
  unit tests in the network module.
- `scripts/`: explicit loopback database management, checks, tests and Trunk.
- `client/dist/`: generated static site, ignored. Database deployment is separate.

The root is a Cargo workspace, defaulting to the client. Browser/WASM is the
primary distribution target; native `cargo run` is a secondary debugging path.
Both SDKs and the CLI stay aligned at 2.14.0; Bevy and Trunk match the reference.

## Connection lifecycle

Bevy Startup builds an anonymous connection. The browser SDK builds asynchronously
via `spawn_local`; native builds use the synchronous SDK builder. Connection
callbacks subscribe to `presence`. Only subscription application marks the client
ready; an open socket alone is insufficient.

Callbacks enqueue events through a weak reference to an attempt-specific inbox.
Reconnect drops that inbox, closes the old connection, and clears stale state.
Late callbacks from abandoned attempts cannot overwrite current state; a late
successful async build is explicitly disconnected. We do not call the SDK's
`run_threaded`: Bevy's Update applies events via `frame_tick`, while the SDK's
transport performs asynchronous I/O.

Presence is derived only from the replicated table. Disconnect/error clears the
displayed list and labels it unavailable, rather than presenting stale clients as
online. IDs are sorted for stable presentation. Anonymous tokens are not logged
or persisted; identity is not an account system.

## Scope and limitations

The database table is public: every subscriber can see connection IDs/identities.
This is a development probe, not a privacy-reviewed production directory.
There are no accounts, movement, maps, counters, gameplay reducers, automatic
retry, persistence of identity, or production hosting automation.

Presence cleanup follows the database's disconnect detection, not a browser
unload promise. A killed browser/network outage can delay removal. Browser
background throttling can delay presentation: use visible windows for QA.
The table is durable storage, but its meaning is active transport sessions, not
historical users. Treat a server crash/restart as an outage; do not use presence
as durable account or gameplay state.

Future features should use server-owned tables/reducers, regenerated bindings,
and a narrow snapshot/intent boundary. Add modules for actual requested domains,
rather than speculative empty gameplay abstractions.

## API evidence

Integration follows the sibling reference and the **resolved 2.14.0** installed
crate source: `spacetimedb-sdk/src/db_connection.rs`, `db_context.rs`,
`subscription.rs`, and generated `DbConnection` builders. The SDK's Cargo manifest
explicitly provides the browser feature and async WASM builder. Bevy 0.20.1's
installed examples/reference establish `Camera2d`, UI `Text`, `FontSize::Px` and
the web canvas window options.

Versioned references:
- https://docs.rs/bevy/0.20.1/bevy/
- https://docs.rs/spacetimedb/2.14.0/spacetimedb/
- https://docs.rs/spacetimedb-sdk/2.14.0/spacetimedb_sdk/
