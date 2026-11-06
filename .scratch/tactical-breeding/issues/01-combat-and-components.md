# Settle combat and its components

Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Status: open
Blocked by: none

## Question

Starting only from the [spec](../spec.md), what combat rules and components are needed for executable, understandable tactical encounters that support custom creatures and meaningful responses to exceptional power?

Agree the encounter format, participants and squad size, spatial model, turn/time structure, legal actions, resource/stat foundations, information visibility, and victory/termination rules. Define the shared execution semantics needed by compositional mechanics: targeting, conditions, effect ordering, references to previous results, timing, and bounded passive-trigger resolution.

These are open questions, not inherited defaults. In particular, no former squad limit, grid, turn timer, AP/MP framework, or victory condition is adopted by this ticket.

## Completion

- Record enough agreed rules and explicit remaining questions to exercise encounters with small authored fixtures.
- Walk through representative interactions, including a conditional chain and a triggered interaction, to expose ambiguous ordering or legality.
- Explain how positioning or other chosen tactical interactions can support different responses; examples illustrate the rules rather than establish a fixed creature roster or prove balance.
- Update the spec with the agreed combat foundation before resolving this ticket.

Do not wait for a genome-expression algorithm. Creature fixtures can exercise combat without selecting the final genetics, inheritance system, application architecture, or implementation.
