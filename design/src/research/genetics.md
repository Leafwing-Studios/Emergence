# Genetics

**Genetics** allows the player to modify species that are [part of their hive](assimilation.md).
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

To unlock mutations, you must accquire resources, and perform research at a dedicated species-specific facility.

Creating strains and changing strain-level settings (other than to select the first mutation for each slot) comes with some cost to avoid thoughtless twiddling.

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
- strains must be visually distinguishable
- this should be done via player-driven color selection
- all combinations of mutations across different slots should be compatible
- incompatible mutations must belong to the same slot
- mutations should typically represent an increase in power over their base version
- unlocking new mutations should be exciting and worth the resource investment
- every upgrade slot should have at least one "generally good" upgrade
- mutations within the same slot should have clear tradeoffs between them
  - [double-edged](../glossary.md#double-edged) upgrades are particularly good for this
