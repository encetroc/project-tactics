# Build a general Rust genome experimentation lab

Labels: needs-triage
Type: task
Status: open

## Goal

Build an offline execution and inspection framework for experimenting with genome representations and interpretations that produce creatures. Use Rust and a Ratatui TUI in the existing repository's Cargo workspace.

The conceptual model is `f(x) = y`: `x` is genome data, `f` is an interpretation, and `y` is a creature (abilities, stats, passives, etc.). Representation plus interpretation determines behavior. The lab hosts experiments; it does not prescribe genetics semantics.

This issue records the agreed design, not an instruction to implement unrequested gameplay in the connected-client probe.

## Motivation and relationship to existing decisions

The current circular-genome candidate in [02-readable-genetics.md](02-readable-genetics.md) is one possible model to test, not the lab's architecture. [10-genome-expression.md](10-genome-expression.md) concerns interpretation design; the lab supplies a place to explore such designs without assuming that candidate is correct.

The lab must support genuinely different genome models and interpretations, including models without ordered sequences, circular relationships, or gene-distance concepts. It should help reject inadequate representations instead of requiring exceptions to preserve them.

Creature production is the initial scope. Producing interesting creatures does not itself prove balance or strategic counterplay.

## Agreed boundaries

### Genome models and genome data

- A genome model defines its structure, validation rules, permitted edits, and supported inspection/editing capabilities.
- Genome data is a concrete input instance, distinct from its model definition.
- Models are constructed separately from the lab, initially as Rust code.
- The lab may edit only explicitly exposed fields or operations, respecting model validation and structural constraints. It must not assume arbitrary field mutation is valid.
- Changes to the representation itself are code changes, not something the initial TUI must author.

### Interpretations

- Each interpretation accepts exactly one genome type.
- One genome type may have many interpretations.
- An interpretation consumes compatible genome data and its parameters and deterministically produces one creature.
- Identical input, parameters, and interpretation version must yield identical output. Any future randomness must have an explicit recorded seed rather than hidden ambient state.
- Interpretations are constructed separately in Rust. The lab exposes only parameters explicitly declared editable by their authors.
- Do not require a scripting language, dynamic plugin loading, shared cross-model input interfaces, or a pipeline editor for the first version. Ordinary typed Rust interfaces and explicit registration are sufficient starting mechanisms.

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
- A definition identifies the genome input, interpretation/version, and parameters; a run records the executed values and resulting creature or failure.
- Each saved successful run owns a snapshot of the actual processed genome data alongside its parameters and creature result. Do not rely solely on a mutable external genome file/reference.
- Preserve historical runs rather than overwriting old evidence when rerunning an experiment.
- Support individual execution, result comparison, and batches in the initial scope.
- A batch continues when an individual case is invalid or execution fails; record/report that case's failure explicitly rather than silently skipping it or aborting unrelated cases.
- Support parameter sweeps: execute across a declared range/set of exposed parameter values and compare the resulting creatures. For example, vary interaction strength from `0.1` through `1.0` for an interpretation that exposes such a parameter. This is structured batch execution, not a balance proof.
- Dev execution and batch failures remain ephemeral under the lifecycle rules below; persistence applies only to eligible release runs.

## Interpretation identity and lifecycle

### Naming and identity

Use only dev and release function naming categories, for example:

```text
example_dev_1
example_dev_2
example_release_1
example_release_2
```

- For now, data association is based on the Rust function name; it is the persistent interpretation identity.
- Renaming is unsupported. Do not introduce a separate stable-ID scheme as an initial requirement.
- Deprecation is metadata on an existing release, not a third naming category. Do not rename a release to mark it deprecated.

### Dev

- Dev interpretations may execute and be inspected.
- Their results/run data are ephemeral: no saved associated run records.
- They may be deleted freely because no persisted run data belongs to them.

### Release

- Release interpretations may execute and save runs.
- Released behavior is immutable. Behavior changes require a new release function/version, including changes introduced through shared helpers or the associated genome model.
- Release functions cannot be deleted until explicitly marked deprecated.

### Deprecated release

- Deprecated releases may still execute and save runs, clearly marked as deprecated.
- Deprecation makes the release eligible for deletion; it does not itself delete anything.
- Explicit deletion removes the interpretation and its associated stored data, including the genome-data snapshots owned by its runs.
- Genome model definitions are distinct from those stored input snapshots; deleting one release's runs is not blanket deletion of the model or another interpretation's runs.

### Enforcement and deletion safety

- Naming conventions alone cannot prevent someone deleting Rust source. Use a persisted release registry and validation checks to detect missing registrations and enforce the lifecycle policy.
- Accidental removal of a registered release must report a missing registration, not silently cascade-delete stored data.
- Deletion must be deliberate and confirmed, showing what associated data will be removed. Deprecation authorizes eligibility, not unattended destructive cleanup.
- Never silently rewrite previously stored run evidence under an existing release identity.

## First interface and repository integration

- Use a Ratatui TUI as the first inspection/editing interface, not a CLI-only report phase or a web-first lab requirement.
- Keep execution/model contracts independent of the TUI so other interfaces can be added later.
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
- Extend workspace checks/tests and CI to cover the new crates: current `scripts/check.sh` and `scripts/test.sh` explicitly target the existing packages and will not automatically exercise them.
- Do not upgrade dependencies or alter networking/schema as incidental lab setup.

## First proof and acceptance criteria

1. Implement two genuinely different genome representations with interpretations, demonstrating that the framework is not a circular-genome UI disguised as a generic framework.
2. Demonstrate multiple interpretations for at least one genome type and prevent incompatible model/interpretation execution.
3. In the TUI, select a model/input and compatible interpretation, perform supported edits, set exposed parameters, execute, and inspect every declared creature-output component.
4. Compare individual outputs and exercise a batch and parameter sweep, including a failing case that does not stop remaining cases.
5. Verify deterministic execution for identical inputs/parameters and the same interpretation implementation.
6. Demonstrate dev results remain unsaved; release runs preserve independent genome snapshots, parameters, and outputs across restarts.
7. Demonstrate lifecycle enforcement: nondeprecated releases are protected, deprecation does not rename/delete them, deprecated releases remain usable, accidental missing registration is detected, and explicit confirmed deletion removes only the selected release's associated stored data.
8. Test the engine separately from the TUI and existing client/server. Verify the actual terminal inspection flow; compilation alone is not evidence of usability.
9. Run `scripts/check.sh` and `scripts/test.sh` after changes and report exact commands, blockers, and unverified behavior. If implementation touches networking/schema, additionally follow the repository's integration/browser requirements.

## Deferred

- Explanation/causal traces and detailed intermediate-value inspection. Valuable, but not a prerequisite for the first interpretations or lab.
- Combat simulation and strategic counterplay verification.
- Balance proofs and automated balancing systems.
- Breeding simulation and evolution mechanics.
- General transformation pipelines or a pipeline editor.
- Rich in-lab authoring of new genome types or arbitrary interpretation algorithms.
- Additional web/CLI interfaces, dynamic plugin infrastructure, and scripting languages unless later justified.

## Comments

Design captured from the user discussion: the central correction was to make the lab a general execution/inspection framework, with separately constructed representations and interpretations and only explicitly supported in-lab edits. The circular-genome candidate must not leak into the generic lab contracts. The final lifecycle choice deliberately uses function names as identity and dev/release naming with metadata-only deprecation, superseding earlier suggestions of three naming states or a separate identity scheme.
