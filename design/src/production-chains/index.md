# Production chains

In a [factory builder](../high-level/genre-mechanics.md), resources are extracted, transformed and ultimately used.
They are extracted as [raw resources](raw-resources.md), are transformed into [intermediates](intermediates.md) are ultimately turned into useful [end products](end-products.md).

The paths that these **items** can travel on is dictated by the [recipes](recipes.md) available to the player.

## Analyzing production chains

When designing production chains for a factory builder (or frankly, any game with crafting),
the most essential thing to consider is the **recipe graph.**

You can construct the recipe graph by placing the items in the game as nodes of the graph, and then drawing an edge from each of the inputs in a recipe to its outputs.
We can also **color** this recipe graph into three parts that roughly correspond to raw resources, intermediates and end products.
Raw resources have no parents, end products have no children*, and intermediates have both.

As an aside, these are actually directed hypergraphs.
Hypergraphs may sound scary, but they're pretty simple: they're simply graphs that connect **nodes** to each other via **edges**, but the edges map betweeen groups of nodes rather than individual nodes.
"Directed" simply means that the edges have a direction to them: you can't turn electricity back into coal!

Unsurprisingly, this gets quite intimidating for most games!
Instead, it's more useful to:

1. Consider smaller parts of the graph at a time.
2. Reason abstractly about the properties of these graphs, and their effect on gameplay.

Let's go through some of the relevant properties of production graphs!

### Number of nodes (vertex count)

This is a simple measure of the complexity of the game's crafting system.
If there are more items to find and make, there's more game to both learn and play.

Increasing this value increases playtime and diversity at the cost of complexity and development costs.

### Number of [connected components](https://en.wikipedia.org/wiki/Component_(graph_theory))

Imagine the recipes as physically connecting various parts of your graph, and then try to pull it apart.
How many parts can you split it into without breaking any of the edges?

The number of connected components is *very* high relative to the number of nodes in most farming games, survival crafters and other simple crafting systems.
This means that you don't need to reason too hard about what else you could use your resources for,
but limits the [cohesion](../glossary.md#cohesion) of the game's various systems and resource trees.

### Distance between nodes

This describes how many steps it takes to get from a desired set of raw resources to an end product.
When considered globally, this is the **graph diameter.**

This is often very low in simple crafting games, capping out at one or two.
Colony sims generally have chains of length one to three,
and even most factory builders end up with a chain length of one to four.

Increasing this has a massive complexity cost, but adds a lot of [depth](../glossary.md#depth-and-complexity).
Increasing this well (from a game-design perspective) requires some combination of:

- simpler, less effective early recipes to slowly teach the player
  - this is the strategy taken by the AngelBob's mod pack for Factorio: iron plates can be produced in many different ways, and only slowly becomes more complex
- good uses for intermediate steps in the chain, either as end products, or via heavy branching
  - this is also used in AngelBob's: sulfur is complex to make but exceedingly useful
- exceptionally good tooling that displays the entire production tree needed for each recipe
- exceptional patient and/or masochistic players
  - this is the strategy taken by [Pyanodon's mod pack for Factorio](https://www.reddit.com/r/factorio/comments/yd1i7l/finally_after_20_hours_of_work_i_present_the/)

### Number of incoming edges

How many different ways can I make this item?
In simple crafting games, this is almost always one, or a class of equivalent goods (different types of metal are common) can be substituted.

Even in factory builders, this can often be quite small: one is common for base gameplay, and even modded rarely has more than three or four options.

When this is 1 everywhere, the recipe graph is a tree.

### Number of outgoing edges

What can I do with this item?
The higher this is, the more options there are for your ingredient.

Items that only have one outgoing edge should always be transformed into their output when possible: there's no alternative use for them.

### [Cycles](https://en.wikipedia.org/wiki/Cycle_(graph_theory))

Does the recipe graph loop back onto itself?
This might represent either a reversible transformation (like water into steam), or a more complex cycling production chain.

Cycles are both interesting and risky: if after one complete cycle you have more of any resource it can be used to amplify that resource.
Note that this is not necessarily a problem!
Recipes take time to craft, and other resources (including power) or time may be depeleted.

But if amplifying cycles are poorly designed, they can become imbalanced, and distort the rest of the economy of the game.

### Ratio of raw resources to intermediates to end products

In simple crafting games, raw resources and end products are abundant, and intermediates are rare.
But in logistics-heavy factory builders (like modded Factorio), intermediates dominate, as they present interesting puzzles.

The absolute number of raw resources cannot become too high in most cases, otherwise it becomes difficult to find the appropriate resources (although a high number of input nodes helps this problem dramatically).

Similarly, increasing the number of end products is quite hard! End products have the highest [complexity budget](../glossary.md#depth-and-complexity)

### Graph constrictions

In more heavily connected recipe graphs, areas where the graph narrows become particularly important.
All possible paths between the two regions must travel through the connected area.

This can be helpful to manage complexity, but can lead to predictable gameplay and increases the importance of narrow graph regions.

Constrictions can be quantified by examining the [vertex connectivity](https://en.wikipedia.org/wiki/K-vertex-connected_graph) and [edge connectivity](https://en.wikipedia.org/wiki/K-edge-connected_graph) of [neighborhoods](https://en.wikipedia.org/wiki/Neighbourhood_(graph_theory)).

## Production chains in *Emergence*

In *Emergence*, we're looking for production chains that are:

- interesting
- robust
- moderately complex

We have a lot of complexity elsewhere, as players must respond to changing conditions.
As a result, simply adding more and more items, and increasing the depth of the production chains is unlikely to be the right choice.

Instead, we want the following key properties:

- many cycles: everything should be able to be broken back down, either physically or via decomposition
- heavy substitutability: powered by graph constrictions to key intermediates
  - raw resources can ultimately be traded off for each other, because you can make the critical intermediates in many different ways
  - similarly, not all recipes must be unlocked
  - example intermediates: fiber, mud, salt
  - intermediates branch back out to produce end products and advanced intermediates
- conservation laws conserve elemental composition: excess elements become waste products
  - nutrient waste fertilizes soil under the building
    - this can be automatically recaptured and distributed via mycorrhizae
  - water waste drains into the soil under the building
  - inorganic waste must be removed
- add complexity through additional restrictions on items rather than long production chains
  - storage challenges
  - transport costs and restrictions
  - spatially constrained production
