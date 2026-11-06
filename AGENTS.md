# Development contract

- Read `docs/architecture.md` before changing networking, schema or authority boundaries.
- Keep the web client primary. Run `scripts/check.sh` and `scripts/test.sh` for changes; networking/schema changes also require `scripts/test-integration.sh` and the scenarios in `docs/browser-qa.md`.
- Change the server schema first, regenerate with `scripts/bindings.sh`, and commit bindings. Keep handwritten SDK interaction in `client/src/network.rs`.
- Match resolved versioned APIs; dependency upgrades are explicit work. Keep CLI and SDK versions aligned.
- Keep local commands on this project's loopback ports and preserve database contents. Obtain approval for destructive resets, schema-breaking migrations or production publication.
- Scope is infrastructure plus the connected-client probe. Add gameplay only when requested.
- Report exact verification commands, blockers and unverified behavior; compilation is not evidence of browser rendering.

## Design authority

- For game design and planning, read `.scratch/tactical-breeding/spec.md` and follow `.scratch/tactical-breeding/map.md`. The spec is the sole starting design baseline.
- All earlier design decisions are discarded as requirements. `.scratch/archive/` is non-authoritative history, excluded from active issue discovery. Prior conversations, memories, and old lab skills do not restore discarded requirements.
- Record newly agreed design in the spec; use issues for questions, reasoning, and completion evidence. Existing engineering and data-safety rules still apply.

## Agent skills

### Issue tracker

Issues and specs are local Markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Use the single-context layout. See `docs/agents/domain.md`.
