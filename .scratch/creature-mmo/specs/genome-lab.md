# Genome Lab specification

Status: agreed revised target design; Rust web-first lab and explicit genome generation are not yet implemented. The TUI design is retired.

## Goal

Build an offline-capable, web-first execution, editing, and visual inspection lab for experimenting with genome generators, genome representations, and interpreters that produce creatures. All authored application code is Rust, including the web UI; this is the project-wide language direction. The web framework is undecided. Extend the existing Cargo workspace.

The forward model is:

```text
D₀ = G(s; p_G)       Generate an initial valid genome
D  = E(D₀)           Apply permitted edits (or leave it unchanged)
Y  = I(D; p_I)       Interpret the actual genome into a creature

Combined: Y = I(E(G(s; p_G)); p_I)
```

- `G`: genome generator; `s`: explicit seed; `p_G`: generator parameters.
- `D₀`: generated genome data; `E`: the applied sequence of rule-respecting genome edits; `D`: actual genome data after editing.
- With no edits, `E` is the identity operation: `E(D₀) = D₀`. The interpreter always receives the genome after the editing stage, including when that stage leaves it unchanged.
- `I`: interpreter; `p_I`: interpreter parameters; `Y`: creature.
- The two parameter sets are independent. A generator is a construction aid, not a replacement for concrete, editable genome data.

The lab hosts experiments; it does not prescribe genetics semantics. JSON may remain a serialization format, but is neither the domain model nor the required editing experience.

This document defines the revised target, not implemented behavior. [Issue 12](../issues/12-genome-lab.md) and [issue 13](../issues/13-genome-lab-editors-and-live-preview.md) are historical TUI implementation records; their Ratatui-specific UI requirements are superseded. The former TUI and its domain engine have been removed. No unrequested gameplay changes belong in the connected-client probe.

## Motivation and relationship to existing decisions

The current circular-genome candidate in [02-readable-genetics.md](../issues/02-readable-genetics.md) is one possible model to test, not the lab's architecture. [10-genome-expression.md](../issues/10-genome-expression.md) concerns interpretation design; the lab supplies a place to explore such designs without assuming that candidate is correct.

The lab must support genuinely different genome models and interpretations, including models without ordered sequences, circular relationships, or gene-distance concepts. It should help reject inadequate representations instead of requiring exceptions to preserve them.

Creature production is the initial scope. Producing interesting creatures does not itself prove balance or strategic counterplay.

## Agreed boundaries

### Genome models and genome data

- A genome model defines its structure, validation rules, permitted edits, and supported inspection/editing capabilities.
- Genome data is a concrete input instance, distinct from its model definition. Version the coded model, not each edit to an input instance.
- Each versioned genome model supplies a meaningful Rust data type. Several models/versions may supply the same type.
- Models are constructed separately from the lab as Rust code.
- The lab may edit only explicitly exposed fields or operations, respecting model validation and structural constraints. It must not assume arbitrary field mutation is valid.
- Changes to the representation itself are code changes, not something the web editor must author.
- Generated genomes remain directly editable. Editors expose meaningful, model-specific operations rather than requiring hand-authored JSON.
- The model owns validation and allowed operations; the generator and editor use the same authoritative rules. Invalid/incomplete drafts may be displayed but must not be interpreted as accepted genomes.

### Genome generators

- A coded Rust generator constructs valid genome data from an explicit seed and exposed parameters, making construction easier and less error-prone than building inputs by hand.
- The same seed, generator version, model contract, and generator parameters must produce the same genome. A seed alone is not a reproducibility guarantee across parameter or version changes.
- Generators may produce a subset of valid genomes. Valid manual edits need not remain reachable through generator inputs.
- Preserve the distinction between generation, concrete genome editing, and interpretation. Changing interpreter parameters does not implicitly regenerate a genome.
- Regeneration must not silently discard manual edits; the precise candidate/replacement interaction remains a UI design decision.
- Record generator identity/version, seed, and effective parameters as provenance alongside the actual edited genome snapshot. An edit log is optional, not required for reproduction.
- No inverse solver is required: do not recover seed/parameters from edited genome data or recover a genome from a target creature. Reading recorded provenance is not inversion.

### Interpretations

- Each interpretation accepts one meaningful Rust input data type initially, not one specific genome model identity. Multiple genome models/versions supplying that type are compatible with the same interpretation.
- One input data type may have many interpretations. Heterogeneous input-type dispatch and implicit conversions are deferred.
- An interpretation consumes compatible genome data and its parameters and deterministically produces one creature.
- Compatible types plus validated genome input and function parameters must yield valid output. A failure after accepted validation is an implementation error, not ordinary incompatibility.
- Identical input, parameters, and interpretation version must yield identical output. Any future randomness must have an explicit recorded seed rather than hidden ambient state.
- Interpretations are constructed separately in Rust. The lab exposes only parameters explicitly declared editable by their authors.
- The interpreter consumes `D`, the actual possibly edited genome, not necessarily the original generated `D₀`.
- Do not require a scripting language, dynamic plugin loading, a universal genome representation, or a pipeline editor. Ordinary typed Rust contracts and explicit registration are sufficient.

### Type-derived compatibility graph

- Represent compatibility as a bipartite graph: genome-version nodes and function-version nodes, with edges only between the two kinds. Many disconnected islands are expected.
- Derive edges from matching coded Rust data contracts; do not maintain a separate manual edge list or infer compatibility from JSON shape, labels, lifecycle, or graph reachability.
- Compatibility requires a direct edge. Sharing an island does not make every genome/function pair compatible.
- Use meaningful types rather than treating every collection of numbers as interchangeable. Enforce the same contract in execution and web UI selection.
- The existing model/interpreter typed-compatibility rules remain the baseline. How generators are registered and linked to interpreters, including whether pairings are exclusive, is not yet decided. Do not silently impose one-to-one pairing or treat matching JSON shape as semantic compatibility.
- Dev/release combinations are all allowed when their types match, including release functions with dev genomes and vice versa. Persistence eligibility is separate.

### Creature outputs

- Interpretations dictate their output format by composing it from a standard set of supported components.
- This is not one prematurely fixed universal stat block. Components must accommodate the declared creature stats, abilities, passives, and other supported creature fields.
- The lab must present all declared output; silently dropping unsupported fields is not acceptable.
- The web UI must support rich, structured inspection rather than text-only result dumps: interactive 3D creature shape, stats, curves, abilities (including range, AOE, and effect chains), and passives as those components are introduced. The component/viewer design must allow further kinds of output.
- Use structured semantic output data to drive appropriate viewers. Text details remain useful as a complement, not the sole presentation.
- Spatial range/AOE previews and effect-chain inspection are visualizations, not evidence of combat simulation or balance. These requirements do not authorize implementing the entire gameplay system in the first web slice.
- Comparisons must use semantic identity and units/meaning, not merely matching display names. Two fields called "power" need not be comparable.
- Initial interpretation output is a creature only. Intermediate transformations may exist privately inside an interpretation, but general transformation composition is deferred.

### Lab responsibilities

- Discover/select registered models, generators, and interpreters and expose only compatible combinations.
- Generate concrete inputs from seeds and generator parameters, load existing inputs, and expose supported direct genome edits and interpreter parameters.
- Validate, execute, inspect creatures, compare results, and run batches.
- Preserve reproducible experiment definitions and eligible run records.
- Remain independent of any particular genetics algorithm or representation.
- Do not couple the engine to Ratatui, Bevy, SpacetimeDB, or network/database startup.

## Experiments, batches, and parameter sweeps

- Treat experiment definitions and recorded runs as distinct concepts.
- A definition identifies the genome model/version, actual genome input, interpreter/version, and interpreter parameters, plus generator/version, seed, and generator parameters when generated. A run records the executed values and resulting creature or failure.
- Save the actual edited genome, not just the generation recipe: a seed and generator parameters alone cannot reproduce manual edits. Retain original generation provenance without implying it directly generates the edited result. Historical records without generator provenance remain valid historical evidence; do not invent provenance for them.
- Live execution produces an unsaved preview. Saving a successful run is an explicit action, permitted only when both genome and function are release versions. Deprecation alone does not remove eligibility. Never automatically append a run for every edit or selection change.
- Each saved successful run owns a snapshot of the actual processed genome data alongside its parameters and creature result. Do not rely solely on a mutable external genome file/reference.
- Preserve historical runs rather than overwriting old evidence when rerunning an experiment.
- Support individual execution, result comparison, and batches in the initial scope.
- A batch continues when an individual case is invalid or execution fails; record/report that case's failure explicitly rather than silently skipping it or aborting unrelated cases.
- Support parameter sweeps: execute across a declared range/set of exposed parameter values and compare the resulting creatures. Distinguish seed variation, generator-parameter variation, and interpreter-parameter variation; interpreter-only sweeps hold the actual genome fixed. Rules for combining generator sweeps with manual edits remain to be designed; do not silently replay edits onto unrelated regenerated data. This is structured batch execution, not a balance proof.
- Any execution involving a dev genome or dev function, and batch failures, remains ephemeral under the lifecycle rules below; persistence applies only to explicitly saved eligible release/release runs.

## Genome and interpretation identity and lifecycle

### Naming and identity

Both genome models and functions use versioned dev/release identities, for example `genome_dev_1`, `genome_release_1`, `function_dev_1`, and `function_release_1`.

- Coded names are persistent data-association keys. Renaming is unsupported; do not require a separate stable-ID scheme initially.
- Deprecation and archiving are separate code-owned metadata, not naming categories or UI/store settings. Do not rename entries to change either flag. Code is authoritative over cached registry flags.
- Genome model definitions are distinct from the concrete input snapshots owned by runs.

### Dev and release

- Dev entries may execute with any type-compatible dev or release counterpart. Any run involving dev remains ephemeral.
- Release/release pairs permit explicit saving, not automatic persistence on execution.
- Released model semantics and function behavior are immutable. Changes require new versions, including changes introduced through shared data types, validation, preparation, or helpers. UI-only improvements do not require new computational versions unless they change those semantics.

### Deprecation

- Deprecation marks an existing release without renaming, hiding, archiving, or deleting it.
- Nonarchived deprecated releases remain selectable and executable, visibly marked deprecated. A release/release run remains saveable even if either release is deprecated.

### Archiving and preservation

- Either a genome or function may be archived directly in code; prior deprecation is not required.
- Archive genomes and functions independently, without cascading to their compatible neighbors or requiring neighbors to be archived first.
- Archived entries are hidden from web authoring selectors, but remain registered. Handle an empty active selector or a genome with no active compatible functions without crashing or selecting an incompatible pair.
- Preserve stored runs, genome snapshots, experiment definitions, and historical identity/metadata. Archiving does not rewrite or delete historical evidence. Saved data referencing archived entries remains inspectable; loading must not silently reactivate them for authoring.
- Remove deletion mechanics, destructive confirmation flows, cascade deletion, and deletion-based eligibility. Archiving is not source removal or a data cleanup operation.
- Retain missing-release registration detection: accidental source/registration removal must be reported, not interpreted as archiving or permission to erase data.

## Web interface and repository integration

- Web UI is the primary interactive lab. The Ratatui solution, terminal layout, keyboard/focus conventions, and mandatory identity-named Ratatui editor modules are retired requirements. Do not maintain TUI feature parity or extend it as the new target.
- The retired TUI and its domain engine are removed. Keep this web-first target independent of the reusable Ratatui starter; no Genome Lab functionality belongs in that starter.
- All authored application code is Rust, including web UI and any backend. No web framework, rendering library, browser-WASM versus local-backend architecture, or new deployment/port arrangement is selected by this spec.
- Keep model, generation, validation, editing operations, interpretation, and persistence contracts independent of the chosen UI framework and game/network startup.
- Provide model-appropriate genome editors, generator controls with explicit seed and parameters, and interpreter parameter editors with discoverable actions and inline validation. Do not duplicate authoritative domain semantics in UI-only models.
- Support the workflow: choose a generator/model, generate a valid genome, inspect/edit it, select a compatible interpreter, adjust its parameters, and inspect the creature.
- Recompute unsaved creature previews after accepted genome edits and interpreter parameter changes; generation changes first produce a new genome candidate. Incomplete/invalid drafts do not execute. Retained results must be visibly stale when they no longer match current inputs or a preview fails.
- Rich component viewers and comparisons replace text-only inspection. Exact screen layout and the first representative visualization slice remain to be designed; do not simply translate the terminal panels into HTML.
- Keep output presentation distinct from parameter editing. Purpose-built editors and component viewers are allowed; no universal form framework or dynamic plugin system is required.
- Preserve explicit saving of the exact successful current preview; asynchronous obsolete results must not replace newer results or become saveable under different inputs.
- Remain usable offline after local setup, without a running game database or game client. Offline does not prohibit a local loopback backend; the runtime topology is undecided.
- Reuse the Cargo workspace. Reassess domain contracts and implementation from this specification rather than depending on the removed legacy engine. Do not couple the lab to the connected-client probe, perform a wholesale directory migration, or choose a frontend framework incidentally.
- Preserve the project's local ports, authority boundaries, database contents, and any retained historical lab store. Read architecture guidance before changing networking/schema. Any unavoidable breaking store migration needs explicit approval; do not reset or fabricate old records.
- Keep workspace checks/tests and CI coverage for lab code. Dependency upgrades are explicit work, not incidental setup.

## First proof and acceptance criteria

1. Demonstrate two genuinely different genome representations with Rust generators and interpreters; the lab must not hard-code circular genetics.
2. Verify deterministic generation for identical seed, generator version, model contract, and parameters; verify deterministic interpretation for the same actual genome, interpreter version, and parameters.
3. In a real browser, generate a valid genome, edit it through model-respecting controls, adjust interpreter parameters, and observe the resulting creature. Invalid drafts must not execute; stale/error states must be visible.
4. Demonstrate generation and editing as separate stages: manual edits remain possible, changing interpretation parameters preserves the genome, and regeneration does not silently destroy edits.
5. Inspect all declared output with appropriate structured viewers, not only text. Stage richer viewers as shape, curves, abilities, range/AOE, effect chains, and passives become available; demonstrate a representative visual slice without claiming unimplemented gameplay.
6. Preserve typed model/interpreter compatibility, including shared-type examples and a different-type island; reject incompatible execution outside the web UI as well. Resolve generator registration/pairing before its implementation.
7. Compare outputs and exercise batches and parameter sweeps, including a failing case that does not abort other cases. Distinguish generator/seed sweeps from interpreter-only sweeps.
8. Preserve explicit release/release saving and ephemeral dev-involved runs under existing model/interpreter lifecycle rules. Saved generated runs retain actual edited genome snapshots, generation provenance, interpreter version/parameters, and output across restarts. Decide generator lifecycle eligibility before implementing generator-backed persistence.
9. Preserve independent code-owned deprecation/archiving, historical inspection, missing-registration detection, and absence of deletion cascades. Test legacy-store compatibility non-destructively with fixtures/copies; never rewrite released historical evidence.
10. Test the engine independently of the web UI and game services. Verify actual browser interaction and rendering; compilation is not evidence of visual usability.
11. For implementation changes run `scripts/check.sh` and `scripts/test.sh`; networking/schema changes additionally require `scripts/test-integration.sh` and the scenarios in `docs/browser-qa.md`. Add lab-specific browser scenarios when the web implementation exists. Report exact commands, blockers, and unverified behavior.

## Open implementation/design decisions

- Rust web framework and rendering tools, including integration of 3D views.
- Browser-WASM execution versus a local Rust backend, file/storage integration, and deployment topology.
- Generator identity/version registration and lifecycle: whether it is separately versioned from the model, how release eligibility applies, and how archived generators affect authoring. Existing model/interpreter policy must not be silently rewritten.
- Generator/interpreter links: exclusive pairing versus reusable compatible pairings/defaults. The user's linked workflow does not yet settle exclusivity.
- Regeneration/candidate replacement UX, treatment of manual edits in generator sweeps, initial screen layout, and the first rich visualization slice.

## Deferred or excluded

- Inverse generation (recovering seed/parameters from a genome) and inverse interpretation (finding a genome for a target creature). Exact historical provenance is retained instead.
- TUI feature parity or further TUI UX investment as the target solution.
- Functions accepting heterogeneous Rust input data types, implicit conversions, and manual compatibility-edge authoring.
- Explanation/causal traces and detailed intermediate-value inspection, unless separately requested.
- Combat simulation, strategic counterplay verification, balance proofs, and automated balancing.
- Breeding simulation and evolution mechanics.
- General transformation pipelines or a pipeline editor.
- In-lab authoring of new genome types or arbitrary generator/interpreter algorithms; these remain coded in Rust.
- Additional interactive interfaces, dynamic plugin infrastructure, and scripting languages unless later justified.

## Decision history

The initial TUI demonstrated general genome experimentation, typed compatibility, shared model/interpreter lifecycle, independent archiving, live previews, and explicit saving. Issues 12 and 13 retain that implementation history, not current UI requirements.

The revised solution retires TUI-first design because genome/parameter editing was obscure and text-only result presentation cannot adequately support the intended spatial and compositional creatures. The target is now a Rust web-first lab with rich inspection and explicit seeded generation followed by editable genome data and interpretation. Framework and runtime topology remain undecided. This revision is documentation only.
