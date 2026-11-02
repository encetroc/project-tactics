# Development contract

- Read `docs/architecture.md` before changing networking, schema or authority boundaries.
- Keep the web client primary. Run `scripts/check.sh` and `scripts/test.sh` for changes; networking/schema changes also require `scripts/test-integration.sh` and the scenarios in `docs/browser-qa.md`.
- Change the server schema first, regenerate with `scripts/bindings.sh`, and commit bindings. Keep handwritten SDK interaction in `client/src/network.rs`.
- Match resolved versioned APIs; dependency upgrades are explicit work. Keep CLI and SDK versions aligned.
- Keep local commands on this project's loopback ports and preserve database contents. Obtain approval for destructive resets, schema-breaking migrations or production publication.
- Scope is infrastructure plus the connected-client probe. Add gameplay only when requested.
- Report exact verification commands, blockers and unverified behavior; compilation is not evidence of browser rendering.

## Agent skills

### Issue tracker

Issues and specs are local Markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Use the single-context layout. See `docs/agents/domain.md`.
