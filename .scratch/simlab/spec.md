# Simlab specification

Status: discussion-derived proposal and decision record; implementation and foundational combat decisions remain open.

## Purpose

Discover how to generatively produce creatures that make tactical combat interesting, understandable, varied, and balanced. Simlab is the working project name.

Work backward from the desired play experience toward a generation method. Do not begin by assuming that genetics, a particular genome representation, or a genome/interpreter architecture is the solution. The underlying question is what creatures and combat rules create worthwhile decisions, and how to generate more of them reliably.

This document records the discussion, including recommendations and alternatives. Recording an option does not select it for implementation. No implementation work, dependency selection, trained-model purchase, or infrastructure provisioning is authorized by this document.

## Relationship to existing work

- [Genome Lab](../creature-mmo/specs/genome-lab.md) explores genome representations, generation, and interpretation. Simlab instead starts with tactical creature design and encounter evaluation. They may eventually connect, but no dependency, merger, retirement, or replacement is decided here.
- The existing player-facing vocabulary is in [CONTEXT.md](../../CONTEXT.md). Use its distinctions between innate definition and combat state, stats and ability parameters, and abilities/passives sharing an effect model.
- Current domain decisions include combat squads of at most three creatures, separate action and movement points, and physical shape that does not itself affect combat. Simlab does not silently change these. Whether size or anatomy should affect combat would require explicitly reopening the physical-shape decision, not treating it as already agreed.
- Simlab is an experimental effort, not authorization to add gameplay to the connected-client probe.

## Core loop

```text
Define combat rules and design goals
                  |
                  v
Create creatures -> validate -> simulate encounters
       ^                            |
       |                            v
       +------ revise <------ evaluate results
                                  ^
                                  |
                           Human playtesting
```

Revision can change a creature, the generator, the test suite, or the available combat mechanics. A generator cannot rescue a combat system that offers no interesting decisions.

There are three distinct execution responsibilities:

1. **Combat simulator:** authoritative code that executes the rules.
2. **Combat agents:** choose legal actions for the participants.
3. **Evaluator:** analyzes outcomes and behavior across encounters.

Creature authorship is a fourth, separate responsibility. Each can use different techniques; scripted systems, search, existing language models, and custom-trained models are not an all-or-nothing choice.

## Desired creature qualities

A visually unique, complicated, or numerically balanced creature is not necessarily enjoyable. Evaluate these qualities separately:

- **Tactical identity:** a clear reason to choose the creature, such as separating enemies, establishing a threatening position, rescuing allies at a cost, or punishing clustering. Stats support identity rather than replacing it.
- **Meaningful choices:** competing benefits that cannot all be obtained at once, rather than many buttons with one obvious best action. For example, one resource can move an ally or pull an enemy, making offense compete with rescue.
- **Counterplay:** opponents can avoid, interrupt, bait, escape, reposition against, or punish the threat. A useful weakness gives the opponent something to do; it is not merely low stats.
- **Contextual value:** allies, opponents, terrain, objectives, and timing change what the creature can achieve.
- **Readability:** players can anticipate threats and consequences without learning an entirely unfamiliar rulebook for every creature.
- **Distinctiveness:** different creatures materially change plans and play patterns, not just numerical outputs.

Predictability and surprise, simplicity and expressiveness, and specialization and broad usefulness are trade-offs to discover through play, not metrics to maximize independently.

## Proposed generation approach

Generate coherent tactical packages rather than independent bags of traits. A useful design hypothesis is:

> Opportunity -> exploitation -> commitment or cost -> opening for response.

Example: a creature anchors near a narrow passage and threatens to pull an enemy. Anchoring prevents movement, allowing opponents to flank, displace it, or contest elsewhere. This is an illustrative mechanic, not an approved combat rule.

Vary what creates the opportunity, how it is exploited, the commitment required, how opponents interfere, and which allies benefit.

Generate limitations alongside strengths rather than adding arbitrary numerical penalties afterward. Illustrative pairings include:

| Strength | Interactive constraint |
| --- | --- |
| Powerful area attack | Telegraph its target area before resolution |
| Long-range control | Require a maintained line or vulnerable anchor |
| Exceptional mobility | Prevent immediate use of its strongest attack |
| Growing battlefield influence | Require contestable setup |
| Strong protection | Protecting one ally exposes another position |

These are candidates to test, not universal restrictions. Fragility alone does not make oppressive denial mechanics enjoyable.

Start with a bounded vocabulary of executable mechanics and expand it when experiments expose a real limitation. Genetics, inheritance, breeding, and biological explanations are not prerequisites for proving the combat-generation approach.

## AI-authored creatures

“Deliberately authored” need not mean authored by a human. An existing language model can propose initial designs and revisions without training a custom model.

### Inputs and outputs

Give the author explicit combat rules, supported mechanics, design goals, and a structured creature format. Candidate mechanics could include movement, damage, healing, shields, displacement, terrain interactions, telegraphs, cooldowns, costs, conditions, and triggers; the actual vocabulary is undecided.

Ask for a tactical interaction rather than merely a cool creature. Require:

- Innate definition: stats, abilities, passives, and their executable parameters.
- Intended tactical identity and player decisions.
- Counterplay and constraints.
- Expected synergies and exploit risks.

Explanations are hypotheses, not proof of quality. New mechanics may be proposed, but remain unexecutable proposals until explicitly implemented and tested. Do not accept invented prose as an executable rule.

### Iteration

```text
AI proposes -> validation -> encounter simulation -> reports
     ^                                                 |
     +---------------- AI revises ---------------------+
```

Keep a diverse collection of promising designs rather than only winners. Optimizing solely for winning encourages oppressive strategies. Preserve strength, interaction, readability, and distinctiveness as separate evaluation dimensions rather than hiding them in one quality score.

## Authoritative simulation

Proposed foundation: a headless Rust simulator with an interface conceptually equivalent to:

```text
Combat state + legal action -> next combat state + recorded events
```

- Ordinary code, not an AI judgment, determines legal actions and their effects.
- Record explicit random seeds and the rule/design versions needed for replay; a seed alone is insufficient across rule changes.
- Agents see only information a player is permitted to know.
- Prefer the same authoritative rules for automated and playable encounters rather than maintaining a separate approximation for evaluation.
- Validate supported effects, targeting, parameter constraints, bounded execution, and resource rules before accepting a design. Also detect dynamic stalls and runaway trigger combinations during execution.
- Keep reproducible encounter definitions and event traces so surprising results can be inspected.

Simulator structure and its relationship to the existing game are not yet selected. Read architecture guidance before any authority, networking, or schema changes.

## Automated combat agents

### Initial options without training

| Agent | Role | Limitation |
| --- | --- | --- |
| Random legal actions | Smoke tests, crashes, rule failures | Not evidence of competent play |
| Scripted heuristics | Cheap, understandable baseline for attacks, position, survival, objectives | Misses unanticipated strategies |
| Search-based agent | Explores combinations and future action sequences | Branching and planning cost can grow rapidly |

Beam search and Monte Carlo tree search are possibilities, not selected algorithms. Turn structure, action space, hidden information, and simulation speed determine suitability.

Use varied policies and strengths. Otherwise evaluation may reward exploiting one bot rather than good creature design. Tests against humans remain necessary to identify important blind spots.

### Optional trained player

If existing agents consistently miss important tactics or search becomes too expensive, investigate a small gameplay policy trained through self-play, not a language model trained from scratch.

A possible loop is observation -> legal-action selection -> battle outcome -> learning update. Train against varied opponents, including earlier policy versions. Prefer one player that handles many creatures over a separate trained model for every creature.

Generalization is more plausible when creatures compose a stable mechanics vocabulary. Arbitrary new executable mechanics, hidden information, long setups, large action spaces, and unfamiliar synergies make the problem harder. Trained players also have blind spots; training does not certify evaluation quality.

## Encounter evaluation

Balance is not an isolated property of a creature. Distinguish:

- **Fairness:** reasonable chances under the intended conditions.
- **Strategic diversity:** several squads and approaches remain viable.
- **Experience quality:** decisions are understandable and enjoyable, including while losing.

Power budgets can filter candidates but cannot establish all three. Situational strengths are useful if players can understand and influence the relevant situations. Whether hard counters are desirable remains open.

### Test suite

Vary allied and opposing squads, maps, objectives, starting positions, first-player advantage, random seeds, and agent strategies/competence. Compare candidate substitution into a reference squad to help distinguish its contribution from its allies' strength. Search explicitly for dominant combinations, not just individually strong creatures.

Measure and report separately:

- Win rates, objective success, matchup extremes, and statistical uncertainty.
- Stalls, repeated lockdown, runaway loops, and noninteractive strategies.
- Unused abilities, repeated action sequences, and dominant openings.
- Opportunities to interrupt, escape, reposition, and punish.
- Whether creatures materially alter squad behavior and tactical plans.

These are diagnostics, not proofs. An unused ability may be poor design or a bot's failure to understand it. False balance occurs when agents cannot exploit meaningful differences. Keep representative failures as regression cases.

### Human evaluation

Use short playable encounters and representative replays to ask:

- Did the player have meaningful choices?
- Could they understand and respond to threats?
- Was losing frustrating or instructive, and why?
- Did the creature feel meaningfully different?

Automation can identify apparently strong, repetitive, exploitable, or unusual creatures. It cannot establish enjoyment from a win rate or from an AI's persuasive description.

## Optional learned evaluator

Do not assume a separate evaluator model is needed.

- **Strength estimation:** start with match statistics from multiple players. Later, a predictor trained on those results could cheaply shortlist designs; unfamiliar mechanics and high-impact decisions still need simulation checks.
- **Decision quality:** observable proxies help, but no single proxy proves interesting play.
- **Human enjoyment:** requires representative human feedback, such as pairwise preferences between encounters. Self-play produces outcomes, not enjoyment labels. A learned preference predictor would initially be an aid reflecting its training feedback, not an authoritative fun judge.

An existing language model can review reports and suggest revisions, but its explanations must be checked against traces and experiments.

## Limitations and confidence

There is confidence that useful automated combat and diagnostic evaluation can be built without training a model. There is no promise that this alone will demonstrate balance or fun.

No-training approaches struggle with large action spaces, long-term setup, hidden information, novel mechanics, and rare squad synergies. Learned approaches add compute, training/debugging complexity, reward exploitation, and their own generalization failures. Neither eliminates human playtesting.

## Indicative costs, not commitments

The discussion gave the following rough planning ranges for a custom gameplay policy, assuming an existing fast simulator and a bounded mechanics vocabulary. These exclude building the combat game and do not guarantee playing strength.

| Target | Engineering effort | Experimental compute |
| --- | --- | --- |
| First trainable player and pipeline | Several engineer-weeks | USD 100–1,000 |
| Useful player across varied creatures and squads | A few engineer-months | USD 1,000–10,000 |
| Strong, robust player across a broad evolving game | Many engineer-months or more | USD 10,000–100,000+ |

A modest learned evaluator might require weeks of engineering and hundreds to thousands of dollars in compute once usable labeled data exists. Feedback collection and validation may dominate its cost; a reliable fun evaluator is an uncertain research goal, not a fixed-price deliverable.

Hourly GPU examples checked during the discussion were approximately USD 0.34–0.89 for RunPod RTX 4090 options and USD 1.99–2.79 for Lambda A100 options. Sources: [RunPod](https://www.runpod.io/gpu-models/rtx-4090), [Lambda](https://lambda.ai/pricing). These are time-sensitive illustrative rates, not reserved capacity or a quote. CPU simulation, storage, availability, taxes, and failed experiments add cost. Benchmark simulation throughput and learning needs before producing a project budget.

Engineering and iteration may cost more than early training runs. Self-development converts some expenditure into development time; it does not eliminate the work.

## Proposed first proof

1. Settle enough combat rules and desired decisions to define a small encounter sandbox.
2. Implement a small authoritative mechanics vocabulary and structured creature definitions.
3. Create deliberate baseline creatures, human-authored or AI-authored.
4. Add multiple simple players, replayable encounters, validation, and diagnostic reports.
5. Generate and revise candidates, compare them in varied encounters, and manually play representative cases.
6. Demonstrate a handful of creatures that change players' plans for interesting reasons before pursuing thousands of outputs.
7. Expand mechanics or introduce trained agents only in response to observed limitations.

The first proof should test whether a small vocabulary plus AI authorship can produce genuinely different, enjoyable tactical interactions. Exact acceptance thresholds, interface, implementation scope, and milestones remain to be decided.

## Open decisions for the next discussion

1. **Desired play experience:** which decisions should dominate a typical turn—positioning, combinations, prediction, resource management, or something else?
2. **Encounter format:** turn structure, exact squad size within the existing limit, maps, objectives, information visibility, and battle length.
3. **Creature identity:** what varies, which mechanics are available, and whether generation only composes existing mechanics or may propose new ones for implementation. Any combat-relevant anatomy reopens an existing domain decision.
4. **Meaning of balance:** comparable individuals, cost-adjusted strength, squad balance, desired matchup asymmetry, and tolerance for hard counters.
5. **Acquisition and selection:** drafting, collecting, breeding, procedural discovery, or fixed rosters; access to powerful creatures changes the balance problem.
6. **First experiment:** initial mechanics, generation constraints, reference squads, agent policies, evaluation metrics, human feedback process, and evidence that would justify continuation.
7. **Engineering choices:** simulator integration, execution topology, report/playtest interface, persistence/versioning, AI author provider, and training integration if later justified.

## Implementation guardrails

All authored application code follows the project's Rust direction. No framework, training stack, runtime topology, port allocation, or dependency upgrade is selected here. Reuse repository workflows and keep experimental execution independent of game/network startup where feasible.

Preserve existing data and authority boundaries. Obtain approval for destructive changes, breaking migrations, or production publication. For eventual code changes run `scripts/check.sh` and `scripts/test.sh`; networking/schema changes additionally require `scripts/test-integration.sh` and the scenarios in `docs/browser-qa.md`. Report actual verification and unverified behavior; compilation does not prove rendering, competent play, balance, or fun.
