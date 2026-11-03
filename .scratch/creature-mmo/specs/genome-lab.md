# Genome Lab specification

Status: agreed target design; follow-up implementation pending

## Goal

Build an offline execution and inspection framework for experimenting with genome representations and interpretations that produce creatures. Use Rust and a Ratatui TUI in the existing repository's Cargo workspace.

The conceptual model is `f(x) = y`: `x` is genome data, `f` is an interpretation, and `y` is a creature (abilities, stats, passives, etc.). Representation plus interpretation determines behavior. The lab hosts experiments; it does not prescribe genetics semantics.

This document defines the agreed target design, not a claim that all behavior is implemented. The initial lab was implemented under [issue 12](../issues/12-genome-lab.md), which remains a historical implementation ticket. [Issue 13](../issues/13-genome-lab-editors-and-live-preview.md) tracks the next implementation: typed compatibility, owned editors, shared lifecycle/archiving, and live previews. See the [lab README](../../../tools/genome-lab/README.md) for current app behavior. No unrequested gameplay changes belong in the connected-client probe.

## Motivation and relationship to existing decisions

The current circular-genome candidate in [02-readable-genetics.md](../issues/02-readable-genetics.md) is one possible model to test, not the lab's architecture. [10-genome-expression.md](../issues/10-genome-expression.md) concerns interpretation design; the lab supplies a place to explore such designs without assuming that candidate is correct.

The lab must support genuinely different genome models and interpretations, including models without ordered sequences, circular relationships, or gene-distance concepts. It should help reject inadequate representations instead of requiring exceptions to preserve them.

Creature production is the initial scope. Producing interesting creatures does not itself prove balance or strategic counterplay.

## Agreed boundaries

### Genome models and genome data

- A genome model defines its structure, validation rules, permitted edits, and supported inspection/editing capabilities.
- Genome data is a concrete input instance, distinct from its model definition. Version the coded model, not each edit to an input instance.
- Each versioned genome model supplies a meaningful Rust data type. Several models/versions may supply the same type.
- Models are constructed separately from the lab, initially as Rust code.
- The lab may edit only explicitly exposed fields or operations, respecting model validation and structural constraints. It must not assume arbitrary field mutation is valid.
- Changes to the representation itself are code changes, not something the initial TUI must author.

### Interpretations

- Each interpretation accepts one meaningful Rust input data type initially, not one specific genome model identity. Multiple genome models/versions supplying that type are compatible with the same interpretation.
- One input data type may have many interpretations. Heterogeneous input-type dispatch and implicit conversions are deferred.
- An interpretation consumes compatible genome data and its parameters and deterministically produces one creature.
- Compatible types plus validated genome input and function parameters must yield valid output. A failure after accepted validation is an implementation error, not ordinary incompatibility.
- Identical input, parameters, and interpretation version must yield identical output. Any future randomness must have an explicit recorded seed rather than hidden ambient state.
- Interpretations are constructed separately in Rust. The lab exposes only parameters explicitly declared editable by their authors.
- Do not require a scripting language, dynamic plugin loading, a universal genome representation, or a pipeline editor. Ordinary typed Rust contracts and explicit registration are sufficient.

### Type-derived compatibility graph

- Represent compatibility as a bipartite graph: genome-version nodes and function-version nodes, with edges only between the two kinds. Many disconnected islands are expected.
- Derive edges from matching coded Rust data contracts; do not maintain a separate manual edge list or infer compatibility from JSON shape, labels, lifecycle, or graph reachability.
- Compatibility requires a direct edge. Sharing an island does not make every genome/function pair compatible.
- Use meaningful types rather than treating every collection of numbers as interchangeable. Enforce the same contract in execution and TUI selection.
- Dev/release combinations are all allowed when their types match, including release functions with dev genomes and vice versa. Persistence eligibility is separate.

### Creature outputs

- Interpretations dictate their output format by composing it from a standard set of supported components.
- This is not one prematurely fixed universal stat block. Components must accommodate the declared creature stats, abilities, passives, and other supported creature fields.
- The lab must present all declared output; silently dropping unsupported fields is not acceptable.
- Comparisons must use semantic identity and units/meaning, not merely matching display names. Two fields called "power" need not be comparable.
- Initial interpretation output is a creature only. Intermediate transformations may exist privately inside an interpretation, but general transformation composition is deferred.

### Lab responsibilities

- Discover/select registered models and interpretations and expose only compatible combinations.
- Load/create concrete inputs using declared capabilities; expose supported genome edits and interpretation parameters.
- Validate, execute, inspect creatures, compare results, and run batches.
- Preserve reproducible experiment definitions and eligible run records.
- Remain independent of any particular genetics algorithm or representation.
- Do not couple the engine to Ratatui, Bevy, SpacetimeDB, or network/database startup.

## Experiments, batches, and parameter sweeps

- Treat experiment definitions and recorded runs as distinct concepts.
- A definition identifies the genome model/version, genome input, interpretation/version, and parameters; a run records the executed values and resulting creature or failure.
- Live execution produces an unsaved preview. Saving a successful run is an explicit action, permitted only when both genome and function are release versions. Deprecation alone does not remove eligibility. Never automatically append a run for every edit or selection change.
- Each saved successful run owns a snapshot of the actual processed genome data alongside its parameters and creature result. Do not rely solely on a mutable external genome file/reference.
- Preserve historical runs rather than overwriting old evidence when rerunning an experiment.
- Support individual execution, result comparison, and batches in the initial scope.
- A batch continues when an individual case is invalid or execution fails; record/report that case's failure explicitly rather than silently skipping it or aborting unrelated cases.
- Support parameter sweeps: execute across a declared range/set of exposed parameter values and compare the resulting creatures. For example, vary interaction strength from `0.1` through `1.0` for an interpretation that exposes such a parameter. This is structured batch execution, not a balance proof.
- Any execution involving a dev genome or dev function, and batch failures, remains ephemeral under the lifecycle rules below; persistence applies only to explicitly saved eligible release/release runs.

## Genome and interpretation identity and lifecycle

### Naming and identity

Both genome models and functions use versioned dev/release identities, for example `genome_dev_1`, `genome_release_1`, `function_dev_1`, and `function_release_1`.

- Coded names are persistent data-association keys. Renaming is unsupported; do not require a separate stable-ID scheme initially.
- Deprecation and archiving are separate code-owned metadata, not naming categories or TUI/store settings. Do not rename entries to change either flag. Code is authoritative over cached registry flags.
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
- Archived entries are hidden from TUI authoring selectors, but remain registered. Handle an empty active selector or a genome with no active compatible functions without crashing or selecting an incompatible pair.
- Preserve stored runs, genome snapshots, experiment definitions, and historical identity/metadata. Archiving does not rewrite or delete historical evidence. Saved data referencing archived entries remains inspectable; loading must not silently reactivate them for authoring.
- Remove deletion mechanics, destructive confirmation flows, cascade deletion, and deletion-based eligibility. Archiving is not source removal or a data cleanup operation.
- Retain missing-release registration detection: accidental source/registration removal must be reported, not interpreted as archiving or permission to erase data.

## First interface and repository integration

- Use a Ratatui TUI as the first inspection/editing interface, not a CLI-only report phase or a web-first lab requirement.
- Keep execution/model contracts independent of the TUI so other interfaces can be added later.
- Code a dedicated Ratatui editor alongside each genome model and function, in separate UI modules. Mirror the implementation identity: `genome_release_1_ui` and `function_release_1_ui`.
- A genome UI edits permitted concrete input fields and any exposed generation/preparation parameters. A function UI edits only its function parameters; it does not own result rendering.
- Editors own field rendering, interaction, draft state, and inline validation feedback, while model/function code remains authoritative for validation and the actual values used in execution. Do not duplicate parameter semantics in a parallel UI-only model.
- Start with simple working editors for testing, not a generic form framework. Preserve compact equal-height Genome/Function/Results selectors and lower genome/function editor areas. The host owns layout, focus, selection, execution, and persistence.
- Recompute an unsaved preview on valid genome edits, genome/function parameter changes, and compatible selection changes. Incomplete or invalid drafts do not execute. Clearly identify any retained previous result as stale when it no longer represents current inputs, including after a preview failure.
- Result rendering remains separate; custom per-function result UIs are deferred. Continue displaying all declared output through the existing generic presentation.
- The repository already is a Cargo workspace (`client`, `server`); extend it rather than introducing another monorepo framework or doing a wholesale directory migration.
- A possible additive layout is:

```text
client/                 # Existing web-first connected-client probe
server/                 # Existing SpacetimeDB module
crates/genome/          # Model/interpretation/execution contracts and logic
tools/genome-lab/       # Ratatui application
experiments/genomes/    # Saved example inputs/experiment fixtures
```

These new paths are suggestions, not fixed implementation requirements. Actual persistence layout and component APIs still need implementation design.

- The lab must run offline without compiling/starting the game's renderer or requiring a running database.
- Preserve the existing web-first client/server workflow, authority boundaries, local ports, and database contents.
- Keep workspace checks/tests and CI coverage for the lab crates. The current `scripts/check.sh` checks both lab packages and `scripts/test.sh` tests them alongside the headless client.
- Do not upgrade dependencies or alter networking/schema as incidental lab setup.

## First proof and acceptance criteria

1. Implement two genuinely different genome representations with interpretations, demonstrating that the framework is not a circular-genome UI disguised as a generic framework.
2. Demonstrate multiple interpretations for at least one data type, multiple genome models sharing that type, and a different-type island. Derive compatibility from types and prevent incompatible execution even outside the TUI.
3. In the TUI, select a model/input and compatible interpretation, edit through their dedicated coded UIs, and observe live previews on valid edits and selection changes. Invalid drafts must not execute; stale results must be marked. Inspect every declared creature-output component.
4. Compare individual outputs and exercise a batch and parameter sweep, including a failing case that does not stop remaining cases.
5. Verify deterministic execution for identical inputs/parameters and the same interpretation implementation.
6. Exercise all four dev/release pairings. Only release/release previews can be explicitly saved; previews never auto-save. Saved runs preserve both version identities, independent genome snapshots, parameters, and outputs across restarts.
7. Demonstrate separate code-owned deprecation and archiving for both genomes and functions: deprecated releases remain usable/saveable, direct independent archiving hides choices without deleting data, missing registrations are detected, and deletion mechanics are absent.
8. Test the engine separately from the TUI and existing client/server. Verify the actual terminal inspection flow; compilation alone is not evidence of usability.
9. Run `scripts/check.sh` and `scripts/test.sh` after changes and report exact commands, blockers, and unverified behavior. If implementation touches networking/schema, additionally follow the repository's integration/browser requirements.

## Deferred

- Custom result UIs; function-owned UIs currently edit parameters only.
- Functions accepting heterogeneous Rust input data types, implicit conversions, and manual compatibility-edge authoring.
- Explanation/causal traces and detailed intermediate-value inspection. Valuable, but not a prerequisite for the first interpretations or lab.
- Combat simulation and strategic counterplay verification.
- Balance proofs and automated balancing systems.
- Breeding simulation and evolution mechanics.
- General transformation pipelines or a pipeline editor.
- Rich in-lab authoring of new genome types or arbitrary interpretation algorithms.
- Additional web/CLI interfaces, dynamic plugin infrastructure, and scripting languages unless later justified.

## Comments

Design captured from the user discussion: the central correction was to make the lab a general execution/inspection framework, with separately constructed representations and interpretations and only explicitly supported in-lab edits. The circular-genome candidate must not leak into the generic lab contracts. Coded names remain identities, with dev/release naming and metadata-only deprecation.

Follow-up design supersedes deletion-based lifecycle rules and one-model-identity-per-function compatibility: genomes now share the function lifecycle, with independent code-owned archiving and no deletion; matching meaningful Rust data types derive a bipartite compatibility graph. Dedicated genome/function editors drive live unsaved previews, and saving requires an explicit action with a release/release pair. Implementation is tracked in [issue 13](../issues/13-genome-lab-editors-and-live-preview.md); this spec update does not implement those changes.
