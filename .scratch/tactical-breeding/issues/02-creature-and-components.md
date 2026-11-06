# Settle the creature and its components

Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Status: open
Blocked by: 01

## Question

Given the [spec's creature and ability baseline](../spec.md#creature-and-ability-baseline) and the newly agreed combat foundation, how should creatures and their components be defined so that structurally varied abilities, passives, and stats can be executed, inspected, and eventually inherited?

Refine the component boundaries, stat definitions versus ability parameters, innate definitions versus combat state, valid conditional chains, targeting/result references, and activation rules. Establish what makes a creature definition valid and understandable without fixing a complete catalogue of spells or creature classes.

## Completion

- Agree the creature/mechanic representation and its boundary with combat execution, including component validity rules and open details.
- Demonstrate structurally different small fixtures using the shared model, including a squad-dependent interaction; do not substitute example kits for the component system.
- State what expression and inheritance must be able to preserve or recombine, without selecting a genome representation simply because an old experiment used it.
- Update the spec with the agreed model before resolving this ticket.

Genome encoding, expression probabilities, breeding controls, numerical balancing, and tool architecture remain downstream unless an explicit decision is necessary to settle this interface.
