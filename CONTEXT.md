# Creature MMO

This context defines the player-facing concepts of the creature MMO.

## Genetics

**Genome**:
A creature's single ordered, circular sequence of inherited genetic elements, whose length can vary between creatures.

**Gene**:
An inherited genetic element whose contribution to a creature's properties depends on its genetic context rather than a fixed combat meaning.

**Influence curve**:
A gene-specific description of how its influence varies with distance along the genome.

**Creature family**:
A breeding starting point characterized by a combat specialization, biome, and appearance, not a permanent class boundary. Different families can interbreed; descendants' possibilities follow inherited material rather than restrictions imposed by a family label.

**Unexpressed genetic material**:
Inherited genetic material present in a creature's genome that contributes no current combat properties.

## Creature composition

**Physical shape**:
A creature's visual form, loosely reflecting its other properties without itself affecting combat.

**Ability**:
A creature mechanic executed when the player explicitly plays it; it uses the same effect model as a passive.

**Passive**:
A creature mechanic executed automatically when its conditions are met; it uses the same effect model as an ability.

**Stat**:
A numerical creature property drawn from common definitions shared by creatures, with applicable limits and value rules, distinct from an individual ability's parameters. A spendable resource is a use of a stat rather than a separate category.

**Innate definition**:
A creature's intrinsic starting properties, including its starting stats and mechanics, distinct from temporary changes during combat.

**Combat state**:
A creature's changing condition within a battle, including current stat values, temporary modifiers, statuses, and position.

**Selective modifier**:
A change to the properties of an individual ability or a group of abilities selected by explicit criteria, rather than an unconditional creature-wide change.

**Ability parameter**:
A property of an individual ability, such as its base range, damage amount, or area, rather than a creature-wide stat. Creature stats can modify applicable ability parameters.

**Effect chain**:
An ordered sequence of effects shared by abilities and passives, with step conditions evaluated against the state left by preceding effects. A failed condition can skip its step or end the chain, as specified by that step.

**Effect**:
A change applied by an ability or passive, such as modifying stats, moving a creature, applying or removing a status, or creating or removing a battlefield entity.

**Status**:
A lasting condition that can carry stat modifiers or triggered effects.

## Combat

**Health**:
A creature's remaining capacity to survive damage; reaching zero supports elimination from combat.

**Action points**:
A creature's spendable budget for playing abilities, separate from its ordinary movement budget.

**Movement points**:
A creature's spendable budget for ordinary grid movement, separate from its ability-use budget.

**Damage modifier**:
A creature stat that modifies damage from qualifying effects.

**Resistance**:
A creature stat that reduces qualifying incoming damage.

**Combat squad**:
The creatures a player brings into one battle; a combat squad may contain at most three creatures.
