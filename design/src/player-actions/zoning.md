# Zoning

Once the player has a set of tiles [selected](selection-tools.md), they may **zone** those tiles, changing what buildings (if any) should be there.

On its face, this mechanic is very similar to standard construction mechanics in RTS or base building games.
The player selects what they want where, and eventually this happens due to the actions of the workers.

However there are three key distinctions:

1. Living structures (plants and mushrooms) can spread on their own accord. After all, they're alive!
2. Zoning can be used to declare that a tile should be kept clear.
3. Zoning persists after a building has been created, and units will attempt to rebuild it if destroyed.

## Signals and priorities

Zoned areas will emit producer or consumer [signals](../glossary.md#signal) if the **contents** of their tile is incompatible with their **target.**
Like all signals, this will build up over time (and eventually) saturate,
causing local units to eventually get to the task.

Like in colony sim games, players can set a **priority** for each tile's zoning.
However, this is not a global priority!
There is no guarantee that all tasks with higher priority will be completed before lower priority tasks begin.
Instead, the zoning priority increase or decrease the [signal](../glossary.md#signal) strength of the zoning.
