# Tactical combat and creature breeding

Status: agreed design direction and scope; detailed rules and implementation remain open.

## Purpose

Design tactical combat with breedable creatures. Use executable models, simulation, evaluation, and human playtesting to discover which implementations deliver the intended experience rather than relying only on intuition.

The goal is not simply to generate varied creatures or balance a fixed roster. Players should be able to pursue custom creatures for a tactical strategy, squad interaction, or response to an opposing strategy. Breeding should open tactical possibilities beyond choosing from prepackaged creatures or classes.

This document records the current direction. It does not authorize implementation, dependency selection, infrastructure changes, or changes to existing data.

## Design authority and fresh start

This document is the sole starting design baseline. All previous design decisions, specs, maps, and issues are discarded as requirements. Only statements explicitly written here carry forward; omission does not imply that an older decision still applies.

The former Creature MMO, Genome Lab, and Simlab planning documents are retained under `.scratch/archive/` solely as historical records. They do not supply defaults, constraints, dependencies, or implementation targets. Previous conversations, memories, and procedural skills describing those designs have no design authority either.

The [new map](map.md) organizes open design work from this baseline. New decisions require discussion and explicit agreement, then an update to this spec; tickets record the reasoning and the map points to it. No old issue is a prerequisite.

This is a planning reset, not permission to delete code or stored data, migrate schemas, select dependencies, or change the existing application's engineering safety rules. Tooling and architecture will follow the new design work rather than a merger or repackaging of the former tools.

## Agreed player experience

### Directed tactical customization

Breeding must let players pursue different tactical possibilities with uncertainty and trade-offs. This is a requirement, not a hypothesis about whether breeding should have that purpose.

A player might want a creature that supports a particular squad strategy or answers opponents who repeatedly disengage. The player should be able to influence the pursuit without necessarily specifying an exact offspring.

Creatures are not limited to permanent predefined classes or a fixed catalogue of complete ability kits. Customization can change mechanics and interactions, not just numerical strength.

The implementation must be tested against these goals. A requirement being settled does not establish that a proposed inheritance system fulfills it.

### Exceptional power is desirable

Players pursuing unusually powerful creatures through breeding is wanted. Do not treat exceptional power itself as a failure or automatically flatten it away.

The desired outcome is exceptional power without universal dominance:

- Several diverse forms of exceptional strength can exist.
- Even the strongest creatures can be defeated through an appropriate counter-strategy.
- Counters may arise from mechanics, squad composition, positioning, or other tactical interactions.
- A powerful creature does not need a compensating intrinsic stat weakness merely because it is powerful.
- A counter relationship need not guarantee victory regardless of play or encounter context.

Rock-paper-scissors is the intuition for competing strengths, not a decision to implement three fixed classes, a literal three-way cycle, or an automatic type-advantage table.

Uncertainty and trade-offs in breeding do not imply a mandatory equal-power budget, hard cap, saturation rule, or a forced weakness attached to every inherited strength. A gene-capacity budget is not part of this baseline and must not be imposed under another name. Finite breeding capacity is a separate question.

### Desired distribution of outcomes

The target is three broad regions, not necessarily three explicit game tiers:

| Region | Desired frequency | Intended character |
| --- | --- | --- |
| Nonviable / very weak | Small minority | Too weak for competitive use |
| Viable / good enough | Large majority | Useful creatures reasonably obtainable through breeding |
| Exceptional / overpowered | Small minority | Unusually strong, diverse, desirable, and strategically counterable |

The user describes this as normal-distribution-like: a large viable middle and small weak and strong tails. This is a desired qualitative distribution, not an established statistical result, chosen probability formula, or exact percentile allocation.

Do not silently turn these regions into mandatory rarity labels, discrete power levels, or a generational upgrade ladder.

Distinguish two questions:

1. How often are different outcomes produced through generation and breeding?
2. How do those creatures perform against different strategies and in different squads and encounters?

One scalar power score cannot establish both. An exceptional creature may dominate many matchups and still lose to an appropriate strategy. A counter does not inherently have to be equally rare or equally exceptional.

## Scope and order of work

The initial ecosystem proposal was too broad. The agreed sequence is:

1. **Settle combat and its components.** Establish the rules and interactions that make encounters executable and understandable.
2. **Settle the creature and its components.** Define what creatures contribute to combat and how their properties and mechanics are represented.
3. **Simulate and evaluate combat, creatures, and breeding.** Test concrete implementations against the intended customization, power distribution, and counterplay goals.

Do not lead with a full simulated player population, economy, acquisition campaign, or evolving metagame. These are not initial requirements.

Working recommendation, not a requirement to freeze each layer permanently: settle enough to experiment, then allow findings in later layers to expose missing rules or prompt explicit revisions. Small authored test creatures may help exercise combat rules; they are fixtures, not a decision to use a fixed final roster.

No suite architecture, screen layout, framework, or application packaging is selected. The research questions should determine the necessary tools, not the reverse.

## Creature and ability baseline

The following statements are explicitly part of this spec's starting baseline. They stand on their own, without incorporating any other requirements from previous decisions, and do not define a fixed ability catalogue.

### Creature composition

A creature comprises:

- Physical shape.
- Stats.
- Abilities.
- Passives.

Physical shape loosely reflects mechanics but has no combat effect itself. Separate innate definitions from changing combat state such as position, current values, temporary modifiers, and statuses.

Creature stats and individual ability parameters are distinct. For example, creature health describes the creature, while base range, area, and damage amount describe an ability. Applicable modifiers can affect either, including selected individual abilities or groups.

All innate abilities and passives are available in combat; selecting an equipped subset is not required. Counts remain open.

### Component-assembled mechanics

Abilities are generated or assembled through a shared mechanic model, not selected only as complete predefined spells. That model includes:

- Activation.
- Requirements.
- Costs and usage limits.
- Targeting.
- Effects.
- Timing.

An ability is explicitly played by the player. A passive activates automatically when its conditions are met. They share the same effect model.

A bounded vocabulary of executable components is compatible with structurally diverse generated abilities. Generated descriptions alone are not executable mechanics.

### Conditional effect chains

Abilities and passives use ordered conditional effect chains:

- Conditions inspect the state after preceding effects.
- A failed condition explicitly skips a step or ends the chain.
- Each step can select independent targets.
- Steps can reference earlier targets and results.

The expandable starting effect families are modifying stats, movement, applying/removing statuses, and creating/removing battlefield entities. Targeting includes creatures, cells, and battlefield entities. Conditions and triggers can inspect the caster, targets, battlefield, combat events, and prior outcomes.

Detailed execution ordering, targeting rules, and safeguards against runaway or infinite trigger chains remain open combat work.

### Inheritance can change structure

Inheritance can preserve recognizable mechanic structures while also recombining internal components. It is not limited to numerical variation of fixed spells.

- Preserve structural validity, including meaningful references to earlier targets or results.
- Do not guarantee that every offspring has self-contained synergy. An ally may supply a condition the creature cannot establish itself.
- Activation can recombine: inherited effects may appear in explicitly played abilities or automatically triggered passives, subject to compatible activation and appropriate combat rules.
- This does not mean converting an ability into a free passive with unchanged strength or frequency.
- Stats, abilities, and passives may interact through expression rather than being independent inheritance tracks.

The exact grammar, inheritance probabilities, player controls, expression rules, and numerical values remain to be designed.

## Simulation and evaluation direction

Evaluate implementations against separate goals rather than one combined quality or fun score:

- **Tactical customization:** can players pursue materially different strategies and responses?
- **Useful variation:** are most produced creatures viable, with the desired smaller tails?
- **Exceptional diversity:** are rare powerful outcomes mechanically and strategically varied?
- **Counterability:** do meaningful responses exist even to exceptional creatures?
- **Inheritance:** does breeding preserve enough recognizable structure to support pursuit while permitting new combinations?
- **Execution validity:** are produced mechanics legal, bounded, and correctly simulated?

Use matchup results alongside any aggregate strength estimate. Evaluate creatures in squad and encounter context, not only isolated duels. Distinguish a creature's weakness from an agent's inability to use it or recognize a response.

Simulation provides evidence about behavior, outcomes, and exploits; it does not by itself establish enjoyment or human readability. Human playtesting remains relevant. No combat-agent algorithm or learned evaluator is selected.

A suggested first focused experiment, not yet a settled implementation plan, is whether players can pursue several answers to one tactical problem while producing useful offspring and avoiding a universally dominant answer.

## Proposals not adopted as initial requirements

The discussion considered simulating complete game ecosystems: breeding campaigns, acquisition costs, limited player knowledge, adaptive populations, and long-term metagames. It also proposed comparing breeding against random acquisition and equal creature access, using intervention experiments and adversarial exploit search.

These remain possible later techniques. The scope was explicitly narrowed before adopting that program. In particular, the current question is not whether tactical breeding should be part of the game; it is how to implement the agreed experience successfully.

## Open decisions

- Detailed combat rules, interactions, and executable mechanic grammar.
- Encounter format, squad size, turn structure, spatial rules, information visibility, victory conditions, and resource/stat foundations. No answers are inherited from the former planning effort.
- Creature composition details and valid component combinations.
- How genomes express structurally diverse abilities, passives, and stats.
- How players influence breeding toward a tactical goal, and what uncertainty remains.
- Inheritance probabilities, breeding limits, and outcome frequencies.
- Operational definitions of viability, exceptional strength, and meaningful counterability for experiments.
- Encounter suites, agents, evaluation methods, and human playtest procedures.
- Tool architecture and any reuse justified by the new design; former tool specifications impose no requirements.

## Decision summary

- Design from the intended tactical-breeding experience, not by repackaging the existing tools.
- Directed tactical customization through uncertain breeding is required.
- Pursuit of rare exceptional power is wanted; universal dominance is not.
- Aim for a large viable middle with small weak and exceptional tails, without assuming a proven normal distribution.
- Abilities follow a compositional mechanic model; complete kits and spells are not fixed catalogue entries.
- Begin with combat, then creature composition, then simulation of combat, creatures, and breeding.
- Full ecosystem simulation is outside the initial scope.
