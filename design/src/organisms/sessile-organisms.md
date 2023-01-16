
# Sessile organisms: plants and fungi

Sessile means "does not move".
In *Emergence*, these organisms may be plants or fungi, and take the role of buildings or machines in other factory builders.

## Growth

Sessile organisms are modelled as automatically producing [assemblers](../glossary.md#assembler), gathering resources from their local environment to load the inputs to their [recipes](../production-chains/recipes.md).

As they complete recipes, they will both produce outputs and typically progress towards the next stage in their [life cycle](life-cycles.md).
These are distinct pools to avoid frustrating complexities around overharvesting of outputs:
simply harvesting all of the available outputs will not harm the organism.

Typically, once a product has been created enough times, the organism will advance to a larger life stage, producing more outputs over time.
This creates an incentive for the players to keep organisms alive for longer time periods,
and creates a satisfying feeling of actually growing plants and fungi.

## Upkeep

Simultaneously, sessile organisms will be able to enter a **wilting** state, caused by failures in nutrients, water or light.
If they go too long without completing a recipe they will wilt.
If they go too long in the wilting state, they will die.

If the output inventory is full, sessile organisms will still craft items, but some fraction of the resources spent will be immediately returned to the soil.
This is subtly distinct from crafting without consuming resources: this mechanism can only occur if there are adequate resources available at all.
As a result, organisms with a full inventory will still have to be in suitable environments: they don't enter a strange stasis.

When wilting, production rates are reduced until a certain number of recipes have been completed,
punishing players for entering this state, and representing the investment needed to repair the damage dealt.

## Harvesting

Once an output has been produced, worker units can perform [work](../glossary.md#work) at this structure, gathering the outputs one at a time.
Gathered outputs become held, and can then be transported to their destination.

## Fertilizing

To add nutrients to a plant or mushroom, units typically amend the soil, rather than providing the nutrients directly to the sessile organism.
This requires [work](../glossary.md#work), and will convert discrete items into their raw forms that are stored in the soil.

## Reproduction

While each sessile organism will have its own reproduction strategy, all will spread on their own.
This can be done by:

- seeds or spores
- vegetative spreading
