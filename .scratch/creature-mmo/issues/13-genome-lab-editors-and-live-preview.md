# Add owned editors, typed compatibility, genome lifecycle, and live previews

> **Historical TUI implementation record; UI design retired.** The [current Genome Lab specification](../specs/genome-lab.md) supersedes the Ratatui-first requirements below with a Rust web-first lab and explicit seeded generation → editable genome → interpretation. Framework selection is open. Preserve this ticket as history, not as a current UI implementation plan; this change does not remove code or stored data.

Labels: ready-for-human
Type: task
Status: ready-for-human

## Goal

Implement the next iteration of the existing offline Genome Lab, following the separate [Genome Lab specification](../specs/genome-lab.md). [Issue 12](12-genome-lab.md) records the already implemented initial lab and is not being revised for this work. This ticket tracks the implementation delta and acceptance checks; the spec defines the target behavior. The documentation change creating this ticket does not implement the feature.

## Current implementation

Issue 13 implementation now provides identity-named genome/function editors, concrete Rust input contracts with typed execution, versioned genome/function lifecycle metadata, independent code-owned archival, and live unsaved previews. `S` explicitly saves a current release/release preview; invalid drafts and preview failures suppress execution and mark retained output stale. Legacy store reads are additive and preserve historical records and inert deletion-tombstone evidence. See the lab README for controls and migration behavior; issue 12 remains historical.

## Required changes

### 1. Dedicated coded parameter editors

- Supply Ratatui UI modules alongside each registered genome and function, named after their coded identity: `genome_release_1_ui`, `function_release_1_ui`, etc.
- Genome editors cover permitted concrete input edits and exposed generation/preparation parameters. Function editors cover function parameters only, not result rendering.
- Editors handle rendering, keyboard interaction, drafts, and inline validation feedback. Model/function validation and computational data remain independent of Ratatui and authoritative; avoid duplicate UI-only parameter semantics.
- Simple working editors are sufficient initially. Preserve the three-column arrangement, compact equal-height selectors, and lower genome/function editor areas.
- The host owns selection, layout, focus, execution, and persistence. Do not introduce a generic form framework or dynamic plugin system.

### 2. Type-derived bipartite compatibility

- Each genome version supplies a meaningful Rust data type; each function version accepts one such type initially.
- Several different genome models/versions supplying the same type automatically work with the same function. Do not require function code to branch on genome identity.
- Derive the compatibility graph from these coded contracts: genome and function versions are the two node sets, edges cross the sets only, and disconnected islands are normal.
- A direct matching contract determines compatibility; shared island membership, matching JSON shape, names, and lifecycle do not.
- Check compatibility in execution as well as selection. Do not add independently authored edges, implicit conversions, or heterogeneous input dispatch.
- Compatible types plus valid genome input and function parameters must produce valid output. Failures after accepted validation are implementation errors, not routine incompatibility.

### 3. Shared lifecycle and independent archiving

- Give genomes the same versioned dev/release identities and code-owned deprecation metadata as functions. Names remain data-association keys; renaming is unsupported.
- Protect released semantics, including shared types, validation/preparation code, and helpers. Behavioral changes require a new release version; presentation-only improvements do not.
- Allow every type-compatible lifecycle pairing: dev/dev, dev/release, release/dev, release/release.
- Keep deprecation separate from archiving. Nonarchived deprecated releases remain visible, marked, executable, and eligible for release/release saving.
- Allow code-owned archiving directly, without prior deprecation, for either genomes or functions. Archive independently without cascading or dependency-order requirements.
- Keep archived entries registered but hide them from authoring selectors. Handle empty selectors and no-active-compatible-function states safely.
- Preserve historical runs, snapshots, metadata, and saved experiment definitions. Historical data remains inspectable; loading an archived reference must not silently reactivate it for editing/execution.
- Remove deletion actions, destructive confirmations, deletion APIs/policies, and active deletion-tombstone behavior. Keep missing-release registration detection for accidental source removal, including archived registrations.
- Preserve existing `.local/genome-lab.json` contents. Introduce an explicit, non-destructive compatibility/migration strategy for legacy unversioned genome identities, old registry data, and deletion tombstones before changing persistence. Do not silently relabel or erase historical evidence; do not pretend previously deleted data can be restored. Seek approval for any unavoidable schema-breaking migration/reset rather than performing one implicitly.

### 4. Live preview and explicit saving

- Recompute an unsaved preview after valid genome input edits, genome/function parameter changes, and compatible genome/function selection changes.
- Incomplete or invalid drafts do not execute. Show validation errors in the editor and mark any retained result stale when it does not represent the current selection/values.
- Preview failures must be visible, with previous results clearly stale rather than presented as current.
- Never auto-save previews. Provide an explicit Save action only for successful release-genome + release-function previews; deprecated releases remain eligible.
- Do not save stale results under current selections/parameters. Persist the exact preview's identities, effective input snapshot, parameters, and output.
- Runs involving either dev entry remain ephemeral. Keep experiment definitions distinct from run records.
- Keep the TUI responsive; scheduling/debouncing is an implementation choice, not a new user-facing requirement. If asynchronous execution is used, obsolete results must not replace newer previews.

## Acceptance criteria

1. Demonstrate simple dedicated editors for the example genomes/functions, including valid and invalid edits, without embedding Ratatui in the engine.
2. Demonstrate two genome model identities supplying one Rust type and sharing a function, plus a separate incompatible type island. Test incompatible execution outside the TUI.
3. Exercise all four lifecycle pairings; only explicit release/release saves persist. Repeated parameter/selection changes must not append runs.
4. Test genome and function deprecation independently and verify deprecated release/release saving still works.
5. Archive either kind directly and independently, including a genome whose compatible function remains active. Verify selectors, empty states, retained historical data, and restart behavior.
6. Confirm no deletion action or cascade remains. Detect missing release registrations rather than deleting their records; preserve archived registrations.
7. Verify live updates after valid edits and selection changes, invalid-draft suppression, stale/error indicators, and saving the exact displayed valid preview.
8. Verify deterministic output and protect released behavior through regression tests. Preserve batches, sweeps, comparison, experiment definitions, and complete generic output inspection under the revised contracts.
9. Test legacy-store handling non-destructively using fixtures/copies, never by resetting the user's store. Document any blocked migration decision.
10. Run `scripts/check.sh` and `scripts/test.sh`, plus actual terminal interaction checks for editors, previews, saving, and archived/empty states. Report exact commands, blockers, and unverified behavior; compilation alone is not UX evidence. Networking/schema changes are not part of this work.
11. Update root and lab READMEs when behavior lands, replacing the explicitly documented current limitations rather than presenting planned behavior as already available.

## Out of scope

- Custom result-rendering UIs; `function_release_1_ui` edits parameters only.
- Functions accepting several unrelated Rust input types, implicit adapters, manual edge editing, or a graph visualization UI.
- Gameplay, networking changes, dependency upgrades, production publication, destructive resets, or data cleanup.

## Comments

Decisions captured from the design discussion: type-based option A was selected; archiving does not replace deprecation; live results are unsaved until an explicit eligible Save action. The prior deletion design is superseded. No further design interview is required before implementation, but implementation-specific persistence hazards must be surfaced rather than silently resolved destructively.

Verification follow-up after independent review: the earlier statement above claimed deliberate mutation checks as red/green evidence. Those were mutation checks, not test-first regression tests, and are not represented as TDD evidence. The overflow regression was written and run red: `scale=1e308` returned `Ok`; reproduction showed `serde_json::to_value(f64::INFINITY)` yields JSON `null` here. Output validation now rejects that invalid component and the focused test passes. The function-switch host regression was also observed red by simulating the old selector behavior (override remained set), then green after selection reset; the isolated PTY now exercises changing the edited `allele_release_1` selection to `allele_dev_1` without an undeclared-parameter preview error. Invalid navigation discards the draft before resuming preview. The self-contained temporary fixture harness `/tmp/issue13-terminal-acceptance.py` builds a temporary registry/executable (no test flags in repo code) and asserts PTY cases for archived experiment load, all-archived with and without history, empty registry with and without history, no active compatible function, comparison, batch continuation, parameter sweep, function override reset, and invalid-draft navigation. ANSI and normalized transcripts plus per-case isolated stores are under `/tmp/issue13-terminal-acceptance/`. `./scripts/check.sh`, `./scripts/test.sh`, and `git diff --check` pass. Changed-path LSP reported zero diagnostics but its server could not confirm clean status (silent-on-clean). No commit, dependency/networking change, or real-store mutation was performed.

Follow-up fixes: corrected result-pane newline rendering and historical metadata indexing when no preview exists; historical inspection now includes genome snapshot, both parameter maps, lifecycle metadata, and output in the no-compatible-function state. `genome_release_5` now has a usable model-validated editor despite having no compatible function. Batch and parameter-sweep execution now carry genome preparation parameters. Backspace/edit keys are scoped away from Results, and experiment loads reset editor selection/draft state. Added regression tests for these seams and genome-parameter batch/sweep behavior. `./scripts/check.sh`, `./scripts/test.sh`, `cargo fmt --all -- --check`, and changed-path LSP diagnostics pass. A fresh isolated PTY smoke test verified editing `genome_release_5` from 1.0 to 2.5; script/transcript/store are `/tmp/issue13-terminal-acceptance.py` and `/tmp/issue13-terminal-acceptance/`. The earlier broader temporary PTY harness referenced above was absent when this follow-up began, so its full scenario suite was not reproduced in this pass. No commit or real-store mutation was performed.

UI follow-up: lower editor/detail panes now receive the yellow focus border when selected with `Tab`; saved results in the normal Results pane are scoped to the active genome/function identity pair. Added tests for editor-focus selection and exact pair filtering. Updated lab controls documentation.
