# Design tactical combat and creature breeding

Labels: wayfinder:map
Status: open

## Destination

Turn the [starting spec](spec.md) into an agreed combat and creature design, then define focused simulation and playtesting of tactical breeding. The result is an executable-design handoff and a bounded first experiment, not a full MMO design or authorization to implement it.

## Notes

- Start fresh. The spec is the sole design baseline; all previous decisions and backlogs are discarded as requirements. Archived documents and remembered agreements supply no implicit defaults.
- Follow the spec's order: combat and its components → creature and its components → simulation and evaluation of combat, creatures, and breeding.
- Resolve design questions with the human. Claim a ticket before work; resolve at most one non-research ticket per session. Follow [tracker operations](../../docs/agents/issue-tracker.md).
- Record agreed rules in the spec, reasoning in the ticket, and a short resolution pointer here. Recommendations, illustrative mechanics, and untested hypotheses remain labeled as such.
- Settle enough to experiment, not every possible rule. Later findings can explicitly reopen earlier decisions.
- Implementation, frameworks, packaging, and dependency selection require separate agreement. Existing code and data are not reset.

## Decisions so far

The [spec](spec.md) is the starting baseline. No tickets in this new effort have been resolved. No old resolutions are carried over by reference.

## Frontier

1. [Settle combat and its components](issues/01-combat-and-components.md) — open, unblocked; start here.
2. [Settle the creature and its components](issues/02-creature-and-components.md) — blocked by 01.
3. [Define the first simulation and evaluation experiment](issues/03-simulation-and-evaluation.md) — blocked by 01 and 02.

Ticket files own status and blockers; this sequence describes the intended work order.

## Fog

- Combat format, spatial and temporal rules, actions, resources, information, victory, targeting, effect execution, and trigger safeguards.
- Creature representation and valid combinations within the compositional baseline.
- Genetic representation, expression, inheritance, player influence, and remaining uncertainty.
- Operational meanings of viability, exceptional strength, and counterability.
- Encounter coverage, agent competence, reproducible evidence, and human playtesting.
- The smallest tools needed to answer those questions, with no inherited suite architecture.

## Outside the initial scope

Full population/ecosystem simulation, economy, acquisition campaigns, long-term metagame simulation, and a complete MMO design. These are not blockers for the initial experiment. No economy, progression, world, or former tool backlog is carried forward.
