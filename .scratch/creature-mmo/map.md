# Find the core of a level-less creature MMO

Labels: wayfinder:map
Status: open

## Destination

Agree a coherent core-game design and the boundaries of its first playable experiment: how tactical combat, genetics, breeding, progression, and a player-driven economy reinforce one another. This is a planning effort, not implementation.

## Notes

Consult the grilling and domain-modeling skills in each decision session. Resolve at most one non-research ticket per session, with the human; claim it before work. Follow docs/agents/issue-tracker.md for tracker operations.

For creature/genetics design, work backward from what the user wants creatures to be. Use the agreed compositional creature system before choosing genetic representation or expression machinery; do not substitute individual example kits for that system. The earlier unsuccessful genetics-first discussion is historical; use the current Answer in the genetic-model decision for the accepted candidate structure and its validation caveats.

Starting constraints confirmed during charting (not resolutions of child tickets):
- Inspirations: Wakfu tactical combat, Dofus player-driven economy, and Axie Infinity creature breeding. These are references, not commitments to copy their systems or monetization.
- Central attraction: an evolving combat meta and player-discovered counters. Genetic combinations should provide depth without overwhelming players; the exact engine of long-term meta change remains undecided.
- Reject XP leveling, repetitive numerical grind, early-creature obsolescence, automatic veteran numerical superiority, and number-gated content. Progression should arise from meaningful actions and new possibilities.
- Baseline combat: one player controls a small creature squad; PvP and PvE should both reward tactical depth. See the combat decision below for squad limits; cooperative encounter structure remains open.
- Breeding should have understandable, percentage-based outcomes: predictable inheritance with some luck. Creatures have limited breeding capacity; exact limits remain open.
- Everyone can pursue every activity. Cooperation and specialization emerge organically through knowledge and efficiency, not exclusive professions. Example: expertise farming a particular creature allows competitive loot prices.
- Players lose nothing on defeat. Do not silently add resource, creature, equipment, or currency loss as an economic sink.
- The original biome-linked family proposal is developed in the genetic-model decision below; wraparound geography, 3D-isometric presentation, recycling methods, land, and construction remain proposals rather than settled requirements.

Child tickets contain their own resolution details. Open tickets are discovered from the issues directory, not listed here. No research tickets are needed yet: the frontier currently consists of design preferences, not external factual prerequisites.

## Decisions so far

- [Define tactical combat and accessible counterplay](issues/01-tactical-counterplay.md): At most 3v3, alternating timed creature turns on a flat grid, visible opponent information, elimination victory, and knowledge-driven counters; genome generation and final action rules are deferred.

- [Define creatures before their genes](issues/11-creatures-before-genes.md): Four-component creature model, shared effects, selective modifiers, and an expandable health/AP/MP/damage-modifier/resistance foundation agreed before genetics.

- [Define a deep but readable genetic model](issues/02-readable-genetics.md): Candidate contextual, variable-length circular genetics agreed without gene capacity; expression must validate the intended length-related tendencies and natural counterplay.

## Not yet specified

- How exploration, wild encounters, capture, loot, and biome-linked creature families form a satisfying daily loop; precise choices depend on combat, genetics, and progression.
- Whether evolving, fusing, releasing, or rerolling creatures serve distinct useful purposes, rather than redundant recycling mechanics; revisit after inheritance and economic demand are defined.
- How content and ecological change sustain discovery once players understand the initial genetic space.
- Social and cooperative encounter structures beyond the single-player squad baseline.
- How the agreed core becomes one coherent design handoff, including worked examples and unresolved validation risks.

## Out of scope

- Implementing gameplay or changing the existing networking foundation during this effort.
- A complete MMO content catalog, exhaustive balancing, production infrastructure, or launch plan.
- Treating blockchain, NFTs, or real-money creature trading as requirements merely because Axie Infinity is an inspiration.
