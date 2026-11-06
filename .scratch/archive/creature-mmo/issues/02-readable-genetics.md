# Define a deep but readable genetic model

Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Status: resolved
Blocked by: 01, 11

## Question

Which creature properties are inherited, how do genes combine into abilities, passives, and stats, and which constraints prevent universally superior creatures while preserving understandable team-building? Establish what players can inspect and predict, using concrete contrasting creatures and a counter-strategy. Decide the role of biome-linked families.

## Comments

### Current override — remove gene capacity

The user explicitly scraps the gene-capacity concept to reduce complexity. Earlier capacity-budget statements below are historical and superseded, including capacity inheritance and capacity-based expression limits. Do not replace it with a renamed budget. Finite breeding capacity is a distinct concept and is not removed. How the genetic system avoids runaway strength without gene capacity remains unresolved.

The user initially provisionally accepted preferred-distance peaks but found them unintuitive. After explanation, the user clarifies the current abstraction: gene influence follows a gene-specific influence curve, whose shape can be anything for now. Do not require preferred-distance peaks, nearby peaks, a bell shape, or a particular falloff/cutoff formula. Curve selection and validation remain downstream.

On interaction strength, the user accepts reinforcement/interference in principle but qualifies the suggested saturation safeguard: bounds must not be forced, extreme results should remain possible, and counterbalance should arise naturally. Do not record mandatory saturation, a hard cap, guaranteed trade-offs, or a replacement capacity budget as agreed. Whether the model actually yields natural counterbalance remains an open validation question.

The user agrees that strategic counterability is sufficient: an extreme strength need not be paired with a compensating intrinsic stat weakness or a forced genetic trade-off. Other mechanics and squad strategies can provide the response. A creature dominant against every available response would be evidence that natural counterbalance has failed, not an accepted outcome. Extreme strength remains allowed; universal dominance is not the design goal.

### Continuation — coherent inheritance with internal recombination

The user agrees that breeding should support both recognizable inherited mechanic structures and internal component recombination, with predictable boundaries. Coherent structures should usually survive, while deliberate internal recombination can produce new mechanics. Which boundaries are preserved, how players influence recombination, and its genetic representation remain open. No specific example chain or probability was adopted.

The user also agrees to mixed protection of component relationships: preserve structural meaning (such as valid references to earlier targets/results), but do not guarantee self-contained synergy. A creature can inherit a conditional benefit without its own way to establish that condition, allowing squad-dependent mechanics and imperfect combinations. The marking illustration establishes this possibility, not a required status or particular ability. Exact validity rules and expression handling remain downstream.

The user agrees that activation is recombinable: inherited effects may appear in either explicitly played abilities or automatically triggered passives, subject to compatible activation and appropriate limits. This does not mean converting an ability into a free passive with unchanged power/frequency. Exact trigger, cost, frequency, and validity safeguards remain for expression and combat decisions. The resistance-reduction illustration is not an adopted mechanic.

The user agrees to cross-family breeding and families as distinctive genetic starting points rather than permanent class boundaries. Descendants' possibilities follow inherited material and can move beyond the original family's specialization; a family label does not permanently prohibit mechanics or combinations. How mixed ancestry is named or displayed remains open.

The user chooses context-dependent genetic meaning over stable component meanings: the same genetic element can contribute different properties depending on other genetic material. Do not assume each abstract symbol has a fixed mapping to an effect, trigger, targeting property, or stat contribution. Which context controls interpretation, how interactions are bounded, and how players inspect them remain open; the genome-to-kit algorithm remains a separate decision.

The user agrees to layered genetic context: mostly local interactions, with some identifiable genome-wide influences. Local combinations should support recognizable inherited patterns; broader influences can alter their expression, with the inspection view explaining relevant dependencies. No particular regulatory-element mechanism or expression algorithm was adopted at this point.

The user subsequently agrees to an ordered genetic sequence: order can change meaning, so the same elements in a different order can contribute different properties. Recognizable stretches can preserve local context through inheritance; inspection should explain meaningful stretches rather than require players to decode raw symbols. Gene count, alphabet, recombination probabilities, and the exact expression algorithm remain undecided.

The user agrees to one combined ordered sequence initially: offspring inherit stretches from both parents into a single sequence, rather than retaining paired parental versions. Unexpressed inherited material can remain within that sequence. Paired-copy genetics is not required by the initial model. How stretches are selected or recombined remains for breeding/expression decisions as appropriate.

The user agrees to one integrated contextual genetic system for stats, abilities, and passives rather than largely independent stat/mechanic inheritance tracks. A genetic change may affect multiple aspects of a creature, producing breeding trade-offs; this does not require every change to affect everything. Relevant cross-property dependencies must be inspectable. Exact mappings and dependency bounds remain for genome expression.

The user agrees that physical shape combines inherited visual ancestry with loose influence from expressed properties. Crossbreeds can retain recognizable ancestry while reflecting aspects of their actual stats/mechanics. Appearance is suggestive rather than a reliable substitute for inspection, and supplies no combat properties. The encoding and visual-generation rules remain unspecified.

### Continuation — variable length and distance-based interaction

The user proposes variable genome length with a distance-based interaction curve: strongest influence from adjacent elements, decreasing influence with distance, and no direct interaction beyond a cutoff. Short genomes could allow repeated self-interaction at different intensities. The user subsequently agrees to explore a circular ordered genome as the interpretation: influence can wrap around and encounter the same element multiple times within the interaction radius. This is the chosen direction for exploration, not a validated expression algorithm; the exact curve, radius, and aggregation rules remain open.

The user agrees that short-genome specialization/rigidity and long-genome versatility/flexibility concern both combat breadth and breeding flexibility, as tendencies rather than guarantees. The intended tendency is narrower reinforced mechanics and more coupled changes in short genomes, versus broader tactical tools and more independently changeable regions in long genomes. Distance falloff alone has not been shown to produce these outcomes. Fixed genome length was an assistant recommendation, not adopted. Variable length is the user's preferred direction.

The user clarifies that influence should follow a curve whose shape depends on the gene, illustratively bell-like. A single universal curve is therefore not adopted. Whether different genes' curves have different reach/cutoffs, and whether any separate genome-wide exception remains necessary, still needs clarification; do not treat a bell-curve illustration as a chosen formula.

The user agrees that repeated interaction may affect both which properties emerge and their numerical strength, with all resulting combat properties constrained by gene capacity. Repetition must not supply free power outside the capacity budget. Capacity remains separate from genome length; neither longer sequences nor repeated self-interaction establish an automatic combat-strength ranking.

### Prerequisite — creature outcomes before genetic machinery

[Define creatures before their genes](11-creatures-before-genes.md) is now resolved. Its compositional system, shared mechanics, and expandable core stats guide this decision; individual examples are not the organizing model. Work backward from that system rather than inferring desired creatures from a proposed gene model. Historical comments below predate this resolution: AP and MP are now adopted as separate budgets, while WP is not.

### Discussion — genetic score, families, and visibility

- The user proposes a gene score usually around 80%, rarely approaching 100%, with stronger combat properties consuming more gene percentage. Examples include more MP/AP/WP and better ability range. Whether this is a capacity budget, expenditure, or another measure remains to be clarified; MP/AP/WP is not yet an adopted combat framework.
- A family is a starting point for breeding, with its own specialization, biome, and appearance. Example: a savanna family specializing in long range. Cross-family breeding and the persistence of family identity are not yet decided.
- The user agrees to fully inspectable genetics, including unexpressed inherited traits, with a simple combat summary and an optional detailed genome view.
- The user requests more precise questions and more examples when discussing how genes generate abilities.

### Discussion — capacity and compositional genetics

- Gene score means capacity budget, not a multiplier on every property. It represents potential, not a ranking of actual combat strength.
- Synergy must be a major source of strength: between abilities, passives, stats, and other creatures. A lower-capacity creature can outperform a higher-capacity creature through better synergy. Whether a higher-capacity creature can reproduce a lower-capacity build and add upgrades remains an open stress test.
- The user chooses component-assembled abilities (model C), potentially with even finer granularity. Genes affect stats and passives as well as abilities. The exact component grammar, allocation rules, and relationship between inherited genome and expressed build remain undecided.

### Discussion — abstract representation and expression boundary

- The combat kit is born with the creature. Later evolution should permit small but significant changes that can make a promising but awkward creature playable; its mechanism remains undecided.
- Inherited properties are capped by gene capacity. Unexpressed genetic material can remain in the genome without affecting stats, abilities, or passives. Selection/priority rules are not yet decided.
- The user wants abstract genetic symbols, illustratively 40 genes encoded as letter-digit pairs such as R5, A9, C1. These counts and symbols are examples, not adopted representation rules. This ticket should settle the representation that enables intuitive breeding and gene expression, not the expression algorithm itself.
- The user explicitly requests a separate ticket for the genome-to-combat algorithm; see [Define genome expression into a combat kit](10-genome-expression.md).
- Exact upgrades are theoretically possible. Higher-capacity parents should be more likely to produce higher-capacity offspring; probabilities belong to the breeding ticket.

### Discussion — step back to biological and game precedents

- The user confirms capacity is separate from the genome, rejects the direction of the proposed slot/token questions, and requests a quick study of real genetics and breeding plus other games before choosing a simplified representation. Slot structure, paired copies, and letter/digit semantics remain unaccepted.
- Research asset: [Real genetics and game breeding: quick design study](../research/genetics-and-breeding.md). Covers biological inheritance, linkage, expression, and interactions; compares first-party descriptions of Axie Infinity, Niche, and Wobbledogs. Its design suggestions are proposals, not resolutions.

### Historical session wrap-up — unfruitful; no model agreed

The user judged this session unfruitful and asked to stop. Neither the slot/token questioning nor the subsequent interaction-based substance/delivery/response examples found a satisfactory direction. Exploring an interaction-based model was permission to explore, not approval of that model. Do not treat any assistant-proposed representation, biological mechanism mapping, example creature, or expression rule as accepted.

The discussion above preserves explicit user statements as context, but does not constitute a resolved genetic design. The representation and expression model remain undecided. The research note is a reference asset, not a design handoff. No Answer was recorded, no decision was added to the map, and no dependent ticket was unblocked. The claim is released so a future session can restart this decision without inheriting the unsuccessful approach.

## Answer

Resolved through live discussion and explicit confirmation of the candidate structure. This Answer supersedes earlier discussion where it conflicts; it approves a design direction, not a validated expression algorithm.

### Genetic structure and inheritance

- A creature has one combined, ordered, variable-length, circular genome, inheriting stretches from both parents rather than retaining paired parental copies. Exact length limits, alphabet, gene repertoire, and encoding are not selected.
- Genetic elements have context-dependent meanings, not fixed mappings to combat components. Order matters. Gene-specific influence curves describe interactions by distance; wrapping around short genomes can produce repeated encounters, including self-interaction at different intensities.
- Curve shapes remain open. Neither preferred-distance peaks, bell shapes, monotonic falloff, a universal cutoff, nor a separate genome-wide influence mechanism is required. The exact handling of distance, direction, repeated encounters, and aggregation belongs to expression.
- Stats, abilities, and passives emerge from one integrated contextual system. A change may affect multiple properties, but need not affect everything.
- Inheritance should usually preserve recognizable mechanic structures while allowing internal component recombination. Preserve structural validity, such as meaningful target/result references, but not guaranteed synergy or self-sufficiency. Teammates can supply conditions a creature cannot establish itself.
- Activation can recombine: inherited effects can appear in explicitly played abilities or automatically triggered passives, subject to valid activation and suitable combat rules. This is not permission to make an ability free with unchanged strength/frequency.
- Families supply distinctive genetic starting points, specialization, biome, and appearance, not permanent classes. Cross-family breeding is allowed; descendants can move beyond ancestral specialization.
- Physical shape combines inherited visual ancestry with loose influence from expressed properties. Appearance supplies no combat properties and is not a substitute for inspection.

### Complexity, visibility, and counterbalance

- Gene capacity is removed, including capacity inheritance and capacity-based expression budgets. Do not reintroduce it under another name. Finite per-creature breeding capacity remains a separate, unchanged constraint.
- Reinforcement and interference may change both emergent properties and numerical strength. No mandatory saturation, compensating weakness, hard power cap, or forced allocation trade-off was agreed. Extreme results are allowed.
- Strategic counterability is sufficient; every extreme strength need not carry an intrinsic stat weakness. Meaningful responses can arise through other mechanics and squad strategies. Universal dominance would indicate failure of natural counterbalance, not success.
- Genetics, including unexpressed material, should be fully inspectable, with a simple combat summary and optional detailed genome view. Explain relevant contextual dependencies and meaningful stretches rather than require raw-symbol decoding. Inspection and prediction must remain usable despite context dependence.
- The combat kit is born with the creature. Targeted later evolution remains a separate progression question; breeding probabilities, controls, costs, and capacity belong to the breeding decision.

### Hypotheses and handoff

Short genomes should tend toward reinforced specialization and more coupled changes; longer genomes should tend toward combat breadth and more independently changeable regions. These are desired tendencies, not guaranteed properties or proven consequences of circularity or influence curves. Natural counterbalance is also unproven.

[Define genome expression into a combat kit](10-genome-expression.md) must choose and test an actual mapping from this structure to the agreed creature model. Use contrasting worked genomes and counter-strategies to test specialization/versatility, valid recombination, inspectability, extreme results, and repeated self-interaction without a capacity budget. No worked numerical genomes or matchups were validated here; they are explicitly handed off, not silently considered complete. Revisit this candidate structure if those tests fail.

[Define predictable breeding and finite breeding capacity](03-breeding-capacity.md) must specify inheritance probabilities, meaningful player controls, variable-length inheritance/recombination, and useful targeted breeding without making randomness the sole path to a counter. No exact probabilities or curve-inheritance mechanism was chosen here.

