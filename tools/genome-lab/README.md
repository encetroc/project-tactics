# Genome Lab

Run offline from the repository root with `cargo run --locked -p genome-lab-tui`. The TUI uses `.local/genome-lab.json` by default; set `GENOME_LAB_STORE` to select another store. It does not start Bevy, SpacetimeDB, or a network service.

## Current implementation

Genome models and interpretation functions have code-owned versioned dev/release identities. The engine derives compatibility from matching concrete Rust input types and executes through those typed contracts; JSON is the editable/persisted representation, not the function dispatch contract. `genome_release_1` and `genome_release_2` share `AlleleInput`; `genome_release_3` supplies a distinct trait-vector type. Incompatible execution is rejected by the engine.

Coded Ratatui editors are kept alongside each example identity. Genome editors expose only model-declared input edits and preparation parameters; function editors edit function parameters, not result rendering. Engine validation and execution remain Ratatui-independent. The three-column selectors remain compact/equal-height, with the genome and function editors below them and complete component output in the generic results view.

Valid input, parameter, or compatible selection changes produce an unsaved preview. Editors keep draft validation errors inline; incomplete/invalid drafts suppress execution and visibly mark retained output stale. Preview failures remain visible and stale previews cannot be saved. `S` saves the current successful preview only for release/release pairs. Deprecation does not prevent execution or saving. Dev-involved runs are ephemeral. Results are never appended as a side effect of preview refresh or redraw.

Archiving and deprecation are separate code-owned metadata. Archived registrations stay in the registry and historical records remain inspectable, but archived models/functions are hidden from authoring selectors and rejected for new execution. Historical result details include the genome identity/snapshot, genome and function parameters, deprecation metadata, and creature output; historical selections are read-only and cannot be saved as the current preview. If every model is archived, the TUI still exposes stored runs for inspection; if there are no runs, it presents an empty history state. Missing registered releases are reported at startup. There are no deletion actions or deletion APIs.

## Controls

- `←` / `→`: focus Genome, Function, or Results. `Tab`: switch between the selector and lower section in the focused column; the focused editor/details pane has a yellow border.
- `↑` / `↓`: move through selector choices or editor fields/parameters. Choosing a genome loads its sample input; choosing a function recomputes a compatible preview. Saved results are filtered to the selected genome/function pair, alongside the current preview.
- In a genome editor, type a numeric value to replace the selected draft and press `Enter` to apply it. Model validation errors stay inline. Press `p` to switch between concrete input fields and exposed preparation parameters. `+` / `-` adjust the selected preparation parameter by `0.1`.
- In a function editor, type a numeric value and press `Enter` to apply it; `+` / `-` adjust the selected parameter by `0.1`.
- `Enter` outside an editor requests a preview refresh. To save a result, select the current preview in Results and press uppercase `S` (Shift+S). This explicitly saves that successful preview only when both the genome and function are releases. Apply valid edits and refresh the preview first if it is stale. Historical results are read-only; dev-involved runs cannot be saved. Lowercase `s` starts a parameter sweep and is not the save command.
- `c`: compare the current preview with a prior compatible run. Components are matched by semantic key; unit mismatches are incomparable.
- `b`: run a batch with valid and invalid cases; failures do not abort other cases. `s`: sweep the selected function parameter.
- `w`, `l`: save/load experiment definitions. Archived or missing references are not silently activated; definitions remain preserved.
- `PgUp` / `PgDn`: scroll result details. `q`: quit.

## Persistence and legacy stores

Store schema changes are additive. A legacy file lacking `schema_version` loads as compatibility version 1; current registrations add schema version 2 and a genome release registry. Old function registry records, runs, snapshots, experiment definitions, unversioned model names, and `deleted_releases` tombstone evidence are retained through load/save. Tombstones are inert historical evidence: they do not block a coded registration, restore deleted data, or enable deletion. Unversioned identities are not silently renamed to versioned identities, and experiments referring to missing/archived entries are not loaded into authoring. Startup detects missing release registrations instead of removing their records.

The migration test uses a temporary fixture. The user's `.local/genome-lab.json` is not reset or cleaned by the tests.

## Code and scope

`crates/genome` owns typed data contracts, model/function validation and computation, lifecycle metadata, run snapshots, comparisons, batches, sweeps, and persistence. `tools/genome-lab/src/*_ui.rs` owns the example editors; the host owns selection, layout, focus, preview scheduling, and persistence. New released computational behavior must use a new coded version; presentation-only editor improvements do not change computation versions.

The [living specification](../../.scratch/creature-mmo/specs/genome-lab.md) defines the target design. [Issue 13](../../.scratch/creature-mmo/issues/13-genome-lab-editors-and-live-preview.md) tracks this implementation; issue 12 remains the historical initial lab ticket.
