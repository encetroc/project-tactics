# Browser acceptance

Start and publish the local DB, then run the web dev server (README).
Use two visible browser windows to avoid background-tab throttling.

1. Open http://127.0.0.1:8082. Wait for `Status: connected`.
   Confirm the Bevy canvas renders text and the accessible status below it agrees.
   The list includes a full connection ID marked `(you)`.
2. Open a second window at the same URL. Both lists must contain the same sorted
   IDs, with different local markers. Count increases by one (other QA sessions
   may already exist).
3. Close that second window. The first eventually removes exactly that ID.
4. Focus the canvas and press R. A new ID replaces your previous ID and only one
   row remains for this browser connection. The reload button also reconnects.
5. Open `/?server=ws%3A%2F%2F127.0.0.1%3A39999`. It must show error/disconnected
   and an unavailable list, not claim an authoritative zero. Open the normal URL
   to recover. Also try `/?server=bad` to check validation before the SDK builder.
6. With the database terminal you own, stop it. Existing clients must clear
   stale rows and report disconnected/error. Restart the same data directory,
   reload clients and confirm live presence returns. Never stop another project's
   database to run this scenario.
7. Run `web-build.sh`, serve `client/dist/` via a static HTTP server, and repeat
   connection + rendering checks without Trunk hot reload.

For automation: `#connection-status[data-state="connected"]` proves subscription
application. `data-count` and full text expose the same snapshot as the canvas.
Use screenshots to establish rendering, not just DOM status. Record browser
console/runtime errors, scenario outcomes and evidence paths under ignored
`.local/qa/`. A passing compiler or SDK test alone is not browser acceptance.
