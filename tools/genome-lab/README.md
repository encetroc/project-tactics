# Genome Lab

Run offline from the repository root with `cargo run --locked -p genome-lab-tui`. The TUI uses a local JSON store at `.local/genome-lab.json`; set `GENOME_LAB_STORE` to override it. It does not start Bevy, SpacetimeDB, or a network service.

## Current implementation

The app is an offline experimentation prototype with two example genome models, multiple interpretations, parameter editing, execution, comparisons, batches/sweeps, and saved experiment definitions. The three-column TUI has compact selectors above generic genome/function parameter areas and a generic result viewer.

Compatibility currently uses an interpretation's model-name string, not shared Rust input types. Functions have dev/release identities; genome models do not yet have versioned lifecycle metadata. Execution is manual and release-function runs are automatically saved. Dedicated coded editors, archiving, and live previews are **not implemented yet**.

## Current controls

- `←` / `→`: move focus between the Genome, Function, and Results columns. The Genome, Function, and Results selectors have matching compact heights; Genome and Function each have a lower parameter area. Scrollbars appear only when choices overflow the available rows, and follow the visible window rather than the selection.
- `Tab`: switch between the upper selector and lower parameter area in the focused column.
- `↑` / `↓`: cycle choices in an upper selector, or select a parameter in a lower area. Changing genome model loads its sample input; the Results column selects a saved or latest result.
- `e`: edit a model-supported field by entering its exposed field path and a numeric value. Invalid edits are rejected by that model.
- `+` / `-`: change the selected genome or function parameter by 0.1. Genome parameters are declared and applied by the selected model; function parameters are declared by the selected interpretation.
- `Enter`: execute; release results are appended with their genome snapshot and parameters, while dev results remain in memory only.
- `c`: compare the latest result with the previous compatible saved release result. Components are matched by semantic key; unit mismatches are reported as incomparable.
- `b`: run a batch containing valid and deliberately invalid inputs to show continued execution and recorded case failures.
- `s`: sweep the selected exposed parameter across a comma-separated set of values.
- `w`, `l`: save/load experiment definitions including model, genome input, genome parameters, interpretation, and function parameters.
- `x`: explicitly confirm deletion of a deprecated release and only that release's stored runs.
- `PgUp` / `PgDn`: scroll the selected result details; `q`: quit.

## Current code-owned deprecation

To deprecate a release, override `fn deprecated(&self) -> bool { true }` in its Rust `Interpretation` implementation. Do not rename it or change its released behavior. The TUI has no deprecation action: startup synchronizes the persisted release registry from code, overriding any older stored deprecation flag. Deprecated releases remain executable and saveable; deletion still requires explicit confirmation with `x`. Existing run snapshots retain their original metadata.

Example model inputs are under `experiments/genomes/`. Rust authors register models and interpretations with `example_lab` (the initial explicit registry). Model-specific validation/editing and interpretation logic live in `crates/genome`; Ratatui is only the front end.

## Agreed next iteration — not implemented

The [Genome Lab specification](../../.scratch/creature-mmo/specs/genome-lab.md) and [follow-up issue](../../.scratch/creature-mmo/issues/13-genome-lab-editors-and-live-preview.md) define the next iteration:

- Dedicated coded Ratatui editors alongside each genome/function, named after their versioned identities, such as `genome_release_1_ui` and `function_release_1_ui`. Genome editors cover permitted input edits and generation parameters; function editors cover parameters only. Custom result UIs are deferred.
- A bipartite compatibility graph derived from meaningful Rust data types: multiple genome models supplying the same type can use the same function. No manual edge authoring or implicit conversion.
- Versioned genomes and functions with dev/release identities, separate code-owned deprecation and archiving, immutable released semantics, and archived entries hidden from authoring selectors while historical data remains intact.
- Removal of deletion mechanics in favor of independent code-owned archiving. The current `x` action still exists until that work lands; it is destructive and is not the planned archival action.
- Live unsaved previews after valid edits and compatible selection changes, inline validation, and clearly marked stale results on invalid input or failure.
- Explicit Save only when both genome and function are release versions. Mixed dev/release pairs may execute but may not save runs; deprecation alone does not block saving.

These are planned changes, not instructions for controls available today. Existing store contents must be preserved during implementation; do not reset the store to adopt the new design.
