# Example Production Chain: Leafcutter Ant Cultivation

This document details a very simple production chain inspired by [leafcutter ants](https://en.wikipedia.org/wiki/Leafcutter_ant).

## Step Zero: Acacacia Grows

*Acacia* plants grow, producing *acacia leaves*.

**Recipe:** 1 sunlight + 1 water + 1 soil nitrogen -> 1 acacia leaf. 10 second craft time.

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

## Step Three Fungal Growth

Like with plants, fungi automatically grow based on the provided inputs.

**Recipe:** 1 leaf (any) -> 1 leuco mushroom. 5 second craft time

## Step Three: Harvesting and Consuming Fungi

Leuco mushrooms are harvested by ants, and then consumed.
