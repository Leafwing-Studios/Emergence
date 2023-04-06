# Example Production Chain: Leafcutter Ant Cultivation

This document details a very simple production chain inspired by [leafcutter ants](https://en.wikipedia.org/wiki/Leafcutter_ant).

See the chapter on [organisms](../organisms/index.md) for general discussion of how organisms grow, reproduce and die.
See the chapter on [logistics](../logistics/index.md) for general discussion about collecting and transporting items.

## Step Zero: Acacia Grows

*Acacia* plants grow, producing *acacia leaves*.

**Recipe:** sunlight + water + soil nitrogen -> acacia leaf

Plants automatically produce this recipe.
Produced items are placed in their output.
When the plant's output is full, it advances to the next **growth stage.**

Plants in higher growth stages produce leaves more quickly, and have a higher output cap.

## Step One: Gathering Leaves

Ants gather *acacia leaves* from wild or cultivated *acacia* plants.

Gathering takes items from the plant's inventory.
The amount of time taken to collect items can be modified by the storage container, the item type, and the worker.

Each ant can only carry a fixed number of items at a time.

## Step Two: Feeding Fungi

Ants take the *acacia leaves*, and transport them to the *leuco* (see [this note](https://en.wikipedia.org/wiki/Atta_sexdens#Fungus_cultivation)) mushrooms.

## Step Three: Soil Amendment

*Acacia leaves* are added to the inventory of the soil tile under the leuco mushrooms.

## Step Four: Decomposition

Items stored in the soil will decompose over time, but leuco mushrooms will cause stored items nearby to decompose faster.

## Step Five: Fungal Growth

Like all [sessile organisms](../organisms/sessile-organisms.md), leuco mushrooms will gather resources.

**Recipe:** soil organic matter -> mushroom chunk

## Step Six: Harvesting Fungi

Excess *mushroom chunk* are harvested by ants, taking it out of their output inventory.

## Step Seven: Consuming Fungi

Stored or carried *mushroom chunks* are consumed by ants when they are hungry.
