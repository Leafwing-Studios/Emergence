# Guided evolution

**Guided evolution** allows the player to modify species that are [part of their hive](domestication.md).
In this menu, players can:

1. Unlock **mutations** for a species.
   1. These can add new options for the species.
   2. These can also dramatically improve the species in some axis to open up a new role for it.
2. Create a new **strain** of a species.
   1. Strains are part of the same species, and share an unlocked upgrade pool.
   2. Each strain can select a different set of mutations.
3. Assign mutations to a specific strain.
   1. Choices are not locked in, but changing them comes with a cost.
4. Configure other strain-level settings, such as signal responses.

The mechanics for assigning mutations are fairly simple:

1. There are some number of slots for each species, typically corresponding to some visible part of their body.
2. Each strain may select up to 1 mutation for each slot.

Unlocking mutations costs biotic mastery for that species, as outlined in [domestication](domestication.md).
Creating new strains and changing strain-level settings (other than to select the first mutation for each slot) comes with some cost to avoid thoughtless twiddling.
The total numbers of strains of each species is limited by your unlocked [technology](technology.md).

## Controlling which strain is produced

Each species has its own way to select which strain of an organism is born.
For sessile organisms, this is selected at construction time, and any natural reproduction will produce organisms of the same strain.
For egg-based units, this is selected at the hatchery, and naturally hatched eggs will produce a random strain.

## Unlocking mutations

Mutations are unlocked by spending biotic mastery (see [domestication](domestication.md)), in a free choice fashion.
Biotic mastery can be banked, up to a generous cap.

Not all mutations can be chosen initially: these are broken down into tiers to gate complexity.
Higher tiers can be unlocked with [technology](technology.md).

## Designing strains

Each species has a small number of mutation slots (3-5), corresponding to a different part of its body.
You can have up to one active mutation in each of these slots.

## Mutation design

Mutations are ultimately about specialization, not improvement.
Each mutation should come with a tradeoff: no mutation should be strictly positive, although all should be power-positive over the base form.
Generally, these should be designed to either solve a problem encountered when trying to use the organism in your factory, or present an oppportunity at the cost of a new problem.
Particularly important problems should be solvable through mutations in different slots to avoid lock-in.

At higher tiers, variations on existing mutations may be found that offer weaker or stronger tradeoffs, or come with an entirely new drawback.

## Realism

This, unsurprisingly, is a wildly "unrealistic" model of genetics and crop / livestock breeding.
It is, however [plausible](../glossary.md#plausibility-and-realism), as it *feels* like genetics and synergizes with the game's aesthetics.

Very briefly, there are some key facts about real genetics, and why they make bad game mechanics:

- genetics involves random mutation
  - yes, and [output randomness](../glossary.md#randomness) is generally frustrating
- most mutations have no effect
  - wow, so exciting
- strains aren't genetically homogenous
  - this is extremely hard to convey to players
  - this is very frustrating at a scale of hundreds to thousands of organisms
  - this is likely to have a significant performance cost
- not all functional mutations are visible
  - yes, but players need to be able to tell what's going on
- cross-breeding is complicated and involves carefully selecting lines and genetic analysis
  - yes, and that's not what this game is about
- traits typically show complex genetic patterns of dominance and recession and polygenetic origins
  - yes, and that's not what this game is about
- eusocial insects have really complicated chromosomes!
  - yes, I know: if it adds gameplay value we can make a nod to that
- eugenics is Very Bad
  - yes!
  - plant and animal breeding is generally accepted as ethical however
  - don't capture people and subjugate them to the hivemind and force them to breed, okay?
    - ~~unless they're into that~~

Again, this game is [*inspired* by biology, not a model of it](../high-level/game-thesis.md).

## Implied Constraints

- each upgrade must be visually distinguishable
  - this follows from the [key design constraints](../high-level/game-thesis.md#key-design-constraints)
  - this may wait until a larger art budget is acquired
- strains must be visually distinguishable
- this should be done via player-driven color selection
- all combinations of mutations across different slots should be compatible
- incompatible mutations must belong to the same slot
- mutations should typically represent an increase in power over their base version
- unlocking new mutations should be exciting and worth the resource investment
- every upgrade slot should have at least one "generally good" upgrade
- mutations within the same slot should have clear tradeoffs between them
  - [double-edged](../glossary.md#double-edged) upgrades are particularly good for this
