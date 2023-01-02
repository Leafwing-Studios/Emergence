# Intent

**Intent** represents the hive mind's collective willpower and focus.
It regenerates over time, and can be spent at a significantly faster rate to influence and temporarily improve allied organisms.

## Using intent

Intent is primarily spent on spell-like **abilities**:

1. Select which ability you would like to use.
2. Use your cursor to target an area.
3. Click and hold, spending intent at a fixed rate in that area.
4. The longer you hold down the mouse button, the more the effect strengthens, or the longer the effect lasts.

Abilities are intended to serve three design roles:

1. Give the players interesting and fun micromanagement tasks to do as they watch their [contraption](../high-level/creative-automation.md) run.
2. Give players tools to deal with crises.
3. Provide an escape hatch for when players want or need more direct control.

Small amounts of intent are also spent on other actions that reflect "collective intelligence":

- [zoning](zoning.md), with higher priority zoning costing more intent
- [terraforming](terraforming.md)
- when altering the [signal configuration](../signals/configuring-properties.md) of a strain
- changing the [recipe](../production-chains/recipes.md) of a building

This is both thematically appropriate and adds a gentle price to micromanagment.
These costs should be very low, and rapidly recovered: only planning or reconfiguring huge swathes of your factory at once should actually require waiting for intent to recover.

[Purely informational hive mind abilities](informational-tools.md) never cost intent (even if thematically it might be defensible) in order to reduce player frustration.

## Abilities

Intent can be used for several abilities, each of which is [unlocked](intent.md#unlocking-and-upgrading) as the player plays the game.
They are listed here in the approximate unlock order.

Abilties are always **dual**: their effect can always be thematically mirrored.
The more straightforward positive effect is always on the left mouse button,
while the negative effect is on the right mouse button.

### *Lure* / *Warning*

*Lure* gathers units to the area.

*Warning* repels them from the area.
This is more efficient to use than Lure.

### *Flourish* / *Fallow*

*Flourish* boosts growth and action rate of all units and buildings in the area.
Costs fo growth and inputs to recipes are unchanged.
Maintenance costs are increased proportionally.

*Fallow* slows down growth and action rate of all unit and buildings in the area.
Maintenance costs are decreased proportionally.
Recover intent for each affected organism for the duration of the effect.
When used in dense areas, Fallow is intent-positive (at the cost of productivity).
It can also be used to slow down consumption in times of need and reduce generation of unwanted products without destroying the organism.

## Unlocking and upgrading

Intent is unlocked at the very beginning of the game, and narratively represents the first "consciousness" of the collective.

Additional abilities are unlocked via the [hive mind upgrade system](../research/hive-mind-upgrades.md) as the player accomplishes [milestones](../glossary.md#milestone), gating complexity behind demonstrated mastery.

Additionally, many organisms have a class of [genetic upgrade](../research/genetics.md) that improves or modifies how abilities influence that strain of the organism.

Similarly, some organisms will have genetic upgrades that interact with various abilities.
Critically, these are never global: for example, organisms that increase your rate of generation or cap or unlock new abilties are forbidden.
Instead, these upgrades are local: they only affect themselves or the nearby area.
For example, upgrades that:

- cause abilities to affect nearby enemy organisms instead
- cause organisms of this strain to go dormant until the next rainfall
- double the duration of all abilties used in a 3 tile radius

are all within the design space.
These have a lower balance risk, require more careful planning and increase the potential for [creativity](../high-level/creative-automation.md) due to their reduced flexibility.
