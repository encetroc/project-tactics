# Define creatures before their genes

Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Status: resolved
Blocked by: 01

## Question

What is the compositional model of a creature, before deciding how genetics produces it? Define the system of components and relationships that can support many creatures and playstyles, rather than designing individual creatures or balancing example kits.

Establish which components describe a creature, what each contains, and how they relate. Review whether physical shape + abilities + passives + stats are the right building blocks or should be improved. Define what a stat is and agree an expandable core stat set within this ticket; do not defer the entire stat foundation. The user's illustrative fields (body parts, combat resources, damage/resistance types, range, areas of effect, and stat changes) are discussion examples, not automatically adopted mechanics.

Work backward from this creature model to requirements for genetics, not forward from gene symbols, slots, or an expression algorithm. Do not require a complete creature catalog or final balance. Inheritance mechanics, genetic representation, and the genome-to-kit algorithm remain separate downstream decisions.

## Comments

### Partial resolution — creature composition and shared mechanics

Historical checkpoint; superseded by the final Answer below.

Recorded at the user's request for continuation in a new session. This is not a final Answer: the ticket remains unresolved, its claim is released, and dependent tickets remain blocked. The agreements below stand, but the overall decomposition is not yet confirmed as complete or optimal.

#### Working creature model

```text
Creature
├── Physical shape
├── Stats
├── Abilities
└── Passives
```

This is the current working decomposition, not a closed decision about the best building blocks.

#### Agreements so far

- Physical shape shallowly reflects a creature's other components/mechanics, but has no actual effect in combat. It is a visual reflection, not a source of combat properties.
- Abilities and passives share an effect model. Their only defining difference is execution: abilities must be explicitly played by the player; passives trigger automatically when their conditions are met.
- Their agreed working structure comprises activation, requirements, cost/usage limits, targeting, effects, and timing. This defines structural fields, not particular costs or combat resources.
- Both support ordered conditional effect chains. Each step specifies whether a failed condition skips that step or ends the chain. Conditions inspect the state after preceding effects, allowing later steps to react to actual outcomes.
- Each effect in a chain can select its own targets and refer to earlier steps' targets and results.
- The starting effect families are: modify stats (damage, healing, resource changes, buffs/debuffs); move (displacement, teleportation, position swaps); apply/remove statuses (lasting conditions carrying stat modifiers or triggered effects); and create/remove battlefield entities (creatures, obstacles, zones). This is an expandable starting vocabulary, not an exhaustive catalog. Conditions, targeting, duration, and chaining describe how effects operate rather than separate effect types.
- Conditions and passive triggers can reference the caster, targets, battlefield, combat events, and outcomes of other effects. Infinite passive-trigger loops must be prevented; exact execution and safeguard rules belong to the later combat-rules decision.
- A creature enters combat with all its innate abilities and passives available, rather than equipping a subset from a larger repertoire. No separate equipped ability/passive loadout is currently required. Counts remain undecided.
- Numerical creature properties stay under one stats model. Each stat defines applicable properties such as current value, limits, and whether it can be spent. Resource describes a stat's use rather than a separate required category.
- Creatures share common stat definitions with different values rather than inventing creature-specific stat vocabularies. Unique mechanics can use statuses and passives.
- The user's earlier HP/AP/MP/WP, damage, and resistance list was illustrative, not an adopted stat catalog. The existing HP-based victory condition from the resolved combat foundation still stands.

#### Still required before resolving this ticket

1. **Review the building blocks:** Are physical shape, stats, abilities, and passives the right decomposition? Can the components or their boundaries be improved? Do not treat the working tree or effect families as proof that the model is finished.
2. **Sharpen the stat model:** Establish what belongs in stats and what properties a stat definition needs. The unified-stats agreement is a starting point, not a complete specification.
3. **Define an expandable core stat set:** Decide the initial stats and their roles here, leaving room for later additions. Do not silently adopt the user's example list or defer the entire core stat decision downstream.

Resume with these open questions and the human's answers. Continue designing a compositional system, not individual creatures or balanced example kits. Genetic representation, inheritance, genome expression, exact combat execution, and numerical balancing remain downstream; they do not need to be solved to complete this ticket.

### Continuation — component boundaries and numerical properties

- The user agrees with the four-component decomposition: physical shape, stats, abilities, and passives. Abilities and passives are two activation modes of a shared mechanic model; statuses are applied lasting conditions rather than a required fifth innate component.
- The user requests a further review for improvements before treating the decomposition as finished.
- Creature stats and ability parameters are distinct. Health or resistance describes a creature; an ability's base range, damage amount, or area describes that ability. Creature-wide modifiers can affect qualifying abilities without requiring all abilities to share their base values. Effects may modify either kind of property.
- Distinguish a creature's innate definition (starting stats and mechanics) from its changing combat state (current values, temporary modifiers, statuses, position). These describe the same creature, not separate creature components.
- Abilities, passives, and effects use a shared targeting vocabulary for creatures, cells, and battlefield entities; detailed targeting rules remain downstream.
- Selective modifiers are supported: passives, statuses, and other mechanics can modify individual abilities or groups selected by explicit criteria, in addition to modifying creature stats. The user confirmed this after clarification. Base ability parameters remain distinct from applicable modifiers; no particular resource, discount rule, or tag taxonomy was adopted by the illustration.
- The user subsequently accepted the stat-definition properties and core stat set recorded in the final Answer below, explicitly allowing later expansion when more combat complexity is needed.

### Approach correction from the user

The goal is a creature system, not specific creatures. Define the model of what a creature is composed of and the relationships between its components, then decide genetics. The assistant's questions about individual mobility, tanking, and example-kit variation missed that intent. The examples below are exploratory context, not the organizing approach for this ticket.

### Creature examples from the user

- A small, fragile creature with sharp eyes: deals damage from a distance and is mobile enough to find ways to hit opponents.
- A larger creature with scales: gets close to enemies, applies debuffs (possibly reducing resistances), and tanks.

These are desired creature examples, not yet agreed families, final kits, or genetic rules. Their limitations and variation remain under discussion.

### Why this ticket comes first

The user judged the previous genetics-first discussion unfruitful and explicitly requested the reverse approach: start with what they want creatures to be, then work genes backward. Neither existing genetics ticket directly resolves this question. Prior explicit user preferences remain context; unsuccessful assistant-proposed models are not accepted requirements.

## Answer

Resolved through live discussion with the user, including explicit approval of the refinements and expandable stat foundation.

### Creature composition

- A creature comprises physical shape, stats, abilities, and passives. Physical shape loosely reflects its mechanics but has no combat effect itself.
- Abilities and passives share one mechanic model; explicit player activation versus automatic conditional activation distinguishes them. Their structure includes activation, requirements, costs/usage limits, targeting, effects, and timing.
- All innate abilities and passives are available in combat; no equipped subset is required. Counts remain open.
- Separate the innate definition from changing combat state: current values, temporary modifiers, statuses, and position are state, not additional innate building blocks.
- Statuses are lasting conditions carrying modifiers or triggered effects, not a fifth required innate component.

### Shared effects and relationships

- Abilities and passives use ordered conditional effect chains. A failed condition explicitly skips a step or ends the chain. Conditions inspect the state resulting from prior steps; each step can select targets and reference prior targets/results.
- Starting effect families: modify stats; move; apply/remove statuses; create/remove battlefield entities. This vocabulary is expandable.
- Conditions and triggers can inspect the caster, targets, battlefield, combat events, and prior outcomes. Exact execution ordering and safeguards against infinite trigger loops remain for the combat-rules decision.
- Abilities, passives, and effects share targeting concepts for creatures, cells, and battlefield entities; detailed targeting rules remain downstream.
- Creature stats differ from individual ability parameters, such as base range, damage amount, and area. Modifiers can affect either. Selective modifiers can affect one ability or groups selected by explicit criteria; no elaborate tag taxonomy is required now.

### Stat foundation

- Stats are numerical creature properties with shared definitions and differing creature values. A spendable resource is a use of a stat, not a separate model; unique mechanics need not invent creature-specific stat vocabularies.
- A stat definition supports an innate starting value, an effective value after modifiers, applicable bounds, and an optional current amount for pool-like stats. Applicable usage rules describe spending, restoration, and resets. Not every stat needs every property; exact recovery and arithmetic rules are not settled here.
- The expandable core comprises health, action points, movement points, damage modifiers, and resistances. Action points budget ability use; movement points separately budget ordinary grid movement. Health supports the existing zero-HP elimination foundation. Damage modifiers and resistances affect qualifying outgoing/incoming damage.
- No physical/magical taxonomy, special third resource, initiative, critical chance, or accuracy stat was adopted. Damage/resistance categories, exact values, costs, recovery, and modifier calculations remain downstream.
- The user explicitly accepts improving or expanding this core later when additional combat complexity calls for it. This is a sufficient foundation, not a claim of exhaustive coverage.

### Handoff

Genetics must produce properties within this agreed creature model, rather than dictate the creature decomposition. No genetic representation, inheritance mechanism, expression algorithm, final balance, or individual creature kit is resolved here. Continue with [Define a deep but readable genetic model](02-readable-genetics.md), then [Define genome expression into a combat kit](10-genome-expression.md). Detailed stat/action behavior belongs to [Complete combat rules around genome-driven abilities](09-complete-combat-rules.md).

