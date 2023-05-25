# Water

Meaningful water is one of the defining elements of *Emergence*.
It is:

- an essential factory resource
- a key aesthetic pillar
- a logistical challenge and opportunity
- defines both macro (biome) and micro-scale environmental variability.

## Prior Art

While water is in a large number of games in the builder meta-genre, it rarely has a real mechanical impact (or behaves in meaningfully water-like ways!).
Instead, it is used:

- for aesthetic effect (*Against the Storm*, *Factorio*)
- as a reskinned resource (*Mindustry*, *Against the Storm*, *Factorio*)
- as a medium for first-person transport (*Raft*, *Stormworks*)

Some games do a better job though: let's focus on a few particularly relevant examples.

### Factorio

Water is a resource. It can be extracted endlessly and rapidly, and shipped huge distances.
It's vital for power production (both initially and in the late game), but in base Factorio it's only used after that for oil production.

Water is also an obstacle, both for the player's factory and the biters. It serves as both a natural defense and limit to factory growth.
This can be overcome by building landfill, a mid-game tech that allows players to replace water with land.

This defense vs expansion tradeoff is by far the most interesting tension in how Factorio uses water.
Players want to build beside water to reduce the cost of defending a base (and have water for steam for power production),
but this will shape how their factory must grow.

Ultimately this doesn't end up mattering a ton, as marginal costs to longer distance transportation of goods is quite low.

Critically though, the utter impassability of Factorio's water severely hamstrings their map design.
Only lakes can exist, as river networks would completely stop the ability for players to ship goods, explore or be attacked by biters.
Islands would be simply unreachable.

This means that interesting potential mechanics (choke points! river shipping! ocean liners! islands with unique resources!) are unexplored,
and water ends up feeling largely cosmetic.

### Sea Block

The AngelBob's mod pack for Factorio, especially in its SeaBlock form, does much more interesting things with water:

- as a thematic element: surrounded by water, you must find a way to make something from nothing
- as a source for hydrogen and oxygen
- as a source of raw resources, by purifying or evaporating water
  - mud, geodes, slag
  - purified, mineralized and salt water
- as a waste output from processes
- as a way to eliminate excess resources by transforming solid waste into a water type and then eliminating via clarifiers
- as an input for renewable plant growth (which then feeds animals or can be used directly to make key goods or power)
- as a solvent for chemicals like sulfuric acid
- to add serious costs (but not spatial constraints) to expanding the factory footprint

### Timberborn

*Timberborn* has by far the most sophisticated water dynamics in the genre to date.
Key features:

- seasonal variability: water flows most of the time, then stops for a modest drought
- steady evaporation
- controls where and when crops can be grown
- beavers must drink water to not die
- flows downstream, and dynamically responds to height changes
- can be stored, either in open pits, via dams or in dedicated storage structures
- irrigation
- some buildings and crops require shallow water
- bridges of limited length to cross water
- limits navigation and complicates logistics

However, it has several key limitations:

- serious aesthetic issues: dried terrain looks horrible
- most of the interesting tools to control water flow (digging holes, pumping water, irrigation) are limited to late game
- drought mechanics are frustrating: limited ability to respond but strong ability to predict, leaving you to slowly watch as your population dies when things go wrong
- immersion challenges: growing crops doesn't take more water, no floods, no rainfall, water from nowhere
- river-centric design with high water sources seems quite fragile: hand-authored maps only

### Oxygen Not Included

In ONI, water is an essential resource. It comes in three forms: water, polluted water, and salt water and is regularly converted back and forth between them.

It is used for:

- sanitation
- crop production (optional)
- oxygen production (optional, but very effective)
- heat exchange
- luxury end products

Water can carry germs, and is routinely recycled as part of gameplay to avoid spreading germs.
Water also has a temperature, and water's role as a heat exchange fluid is quite important in the early game.

Water leaks through some soil types, forming irritating puddles that must be cleaned up.

Water can also be frozen, forming various types of ice (each with their own freezing point!) or boiled, creating steam that can be used to drive engines (or accidentally kill your workers).

For better or worse, the water mechanics in ONI wildly violate conservation laws: it is possible to create water from nothing, and destroy it.
To resolve this, geysers are a key feature, continually producing water. They can however be blocked to limit production.

## Gameplay value of water

Water must:

- be finite but renewable
  - varies heavily by biome
  - some types of water may be scarce while others are abundant
- be an important source of external temporal variability
  - weather
  - seasons
- be an important source of external spatial variability
  - geography shaping water dynamics
  - biomes
- present meaningful barriers to exploration and logistics that can be overcome
  - units cannot cross deep or extended water
  - units move more slowly in shallow water
  - units can cross water via bridges (limited span), terraforming (expensive) or water transport (spiky, complex)
- offer meaningful opportunities for logistics and defense
  - goods can be carried on the water
    - goods should naturally float downriver (implying a water velocity)
    - goods should also be able to be ferried up river or across relatively still bodies of water at higher cost
  - moats! flood traps! navies!
- be able to be meaningfully observed and understood by the player
  - weather and season cues
  - selection details
  - overlays
  - surface water visualization
- be able to be meaningfully manipulated by the player, especially in the mid and late game
  - terraforming allows players to durably shape the landscape, at fairly high time investment
  - as water flows downhill, players must be able to raise water to higher levels reliably
- can be stored
  - surface-area based evaporation mechanics mean that deep holes are effective natural storage
  - lower loss storage should be possible, but expensive
- can be transported
  - canals work great for both irrigation and transport
  - more flexible (upstream, dynamic) mechanisms should be possible but expensive
- reach a stable equilibrium, even as water is added or removed from the system
  - surface-area based evaporation mechanics do an excellent job stabilizing this
  - draining to the ocean / filling up from the ocean also stabilizes this effectively

Water should:

- play other meaningful roles in factory production chains
  - solution and evaporation of solids
  - to create mud
  - cooking
  - washing and purification
  - fertilized water distributed via irrigation
  - circular processing is a key element
- be a useful trigger for conditional effects that players can use to respond to changes
  - seeds that only germinate in water / plants that only become non-dormant when wet
  - goods that begin to float and move when submerged
  - specialized storage that allows stored goods to float when submerged
- be fairly expensive to transport from place to place (to preserve spatial variability)
  - canals are the main exception to this - they require significant investment though
- be somewhat expensive to store for long periods of time (to preserve temporal variability)
- flow downriver even over shallow gradients
  - this creates much more natural river designs
  - implies continuous water height
- flow relatively quickly
- be required by plants to grow
  - roots are the primary mechanism of gathering water
  - not all plants require the same type of water
  - manually watering also works though
- flow through canals and other player-made paths

Water should not:

- create extreme levels of disruption
  - no extreme floods (without very strong tools to mitigate it)
  - stored goods are never washed away
- create disruption that requires constant manual work to respond to

## Aesthetic and versimilitude constraints

Water must:

- create lakes
- create rivers
- create marshes
- move laterally, flowing downhill
- have plausible sources
  - recipes that create more water than they consume should be treated with extreme caution
  - sinks are much less concerning: evaporation is widespread, and water is often incorporated into products
- fill up during the rain
- dry out over time (faster from wider pools)
- be able to support crops on rainfall alone
  - follows from the fact that plants need water
  - WARNING: this does not work reliably with the current set of water mechanics: plants growing on hills generally die
- cause loose goods to float

Water should:

- create waterfalls
  - WARNING: this needs serious design, as it is not at all supported currently
- interact naturally with oceans
- create tidepools
- support tides
- leak out of imperfect holding vessels
- vary by biome
  - in Terraria, weather is dictated by the biome
  - in Minecraft, climate dictates which biomes go where
- meaningfully interact with soil type in plausible ways
- come in different flavors: salt water, muddy water etc.
  - WARNING: this needs serious design consideration, fluid mixing is hard
- carry dissolved / suspended solids
  - WARNING: this needs more design
- cause organisms to drown
  - organisms should only drown when they are overtopped completely
- push light organisms with the current

Water should not:

- have waves
  - this level of simulation is too fine-scale, and will create pointless disruption with poor tools to manage it
- behave erratically (flickering, oscillations, teleporting etc.)
  - visually distracting
  - severly detracts from aesthetics
  - likely to cause weird exploits

## Base water mechanics

*Emergence* uses an unconventional (for a video game) approach to modelling water: ground water dynamics.
This allows for intuitive emergent behavior, good hooks (especially for plant growth!) and a ton of creative power relative to the simplicity of the design.

Water is stored on a per-tile basis.
Water first fills all available pore space in the soil as **surface water**. Above that level, it overflows as **surface water**.
The characteristics of soil and surface water differ dramatically, creating a meaningful (and intuitive) nonlinearity in behavior.
Water characteristics also vary by soil type, allowing for meaningful emergent distinctions between different soil types (and thus biomes).

### Lateral water movement

Water flows from high to low.
The rate at which this flow occurs is proportional to the difference in height of the water column.

The overall effect creates a **flow velocity**, which can be used to transport floating goods, push units and more.

### Floating litter

Litter which is sufficiently light floats on the surface of the water, travelling in a rate (and direction) proportional to the flow velocity.
Each tile may only have one litter pile on it: litter that exceeds the stack size of the item (or is of a different type) piles up.

This effect is slightly randomized to reduce log jams and create a more visually appealing effect.

## Creating water

### Precipitation

Precipitation is similarly simple: on each tile, add water based on the current weather.
This operates to refill water reserves that are far from rivers and oceans.

### Emitters

Emitters are point sources of water, constantly pouring forth from the ground.

Emitters can be produced by players, but these are dramatically weaker (and come at a high cost) relative to built-in emitters.

### Tidal inflow

Water can flow into the world via tides.

This produces huge amounts of water across the entire coast, but the water is only salt water.

## Destroying water

### Crafting

Water can be used by plants to perform photosynthesis.
This is the primary use and sink of water.
Water used in this way is drawn in by roots, which have both an area (typically a radius) and a depth.

Water can also be stored in item form using reusable containers, and carried back and forth.
Water used for crafting must be supplied in this form.
When these containers are emptied, they add water to the tile that they are on.

### Evaporation

Evaporation is simple: water is removed from each tile.
This varies with:

- the presence or absence of surface water
- the soil type (if no surface water is present)
- the light level on each tile (which in turn varies with local conditions, time of day and weather)

Because surface water evaporates at a much faster rate than soil water, this leads to a substantially stable equilibrium with rivers and islands.
As the amount of water increases locally, the rate of evaporation also increases automatically, creating a local balance.

### Drain to ocean

When water flows into the ocean (because the water level is lower there than anywhere else), it is simply destroyed: oceans are very big!

This acts as an ultimate water sink, and avoids flooding the map, even with very powerful rivers.

## Storing water

### Holes

Water can be deliberately stored in holes in the ground.
This is cheap and relatively effective (especially deep, shaded holes) but can be challenging to extract again and make use of.

### In containers

Water can be captured by more expensive, disposable sealed containers, which can then be stored in standard item storage.
This is lossless, but not very dense and quite expensive.

This is intended purely as a buffer for crafting.

### Storage tanks

Storage tanks can store large volumes of water without water loss.

They can be moved, but are super heavy (requiring multiple crabs to move). Unsurprisingly, they're lighter when empty.

## Moving water

### In pots

Basket crabs can choose to wear a heavier earthen pot, rather than a basket.
These are water-tight and can carry fluids.

When a basket crab wearing a pot travels under water, its pot automatically fills.

### In containers

With the help of an organic, consumable sealed containers, water can be transformed into a solid item.
These can be used directly in crafting recipes, but can also be broken, releasing water into the ground.

### In storage tanks

Storage tanks can be shipped using large, dedicated workers in a matter very similar to tanker trains.
This is a great way to transport huge volumes of water, but is significantly challenging logistically and results in bursty transport.

### Canals

Canals are the primary way to move water across the map.
These flow downhill, and while they require significant and disruptive engineering effort, they are entirely passive.
Some water is lost due to evaporation, but this can be substantially mitigated through shading.

### Fountain reeds

Fountain reeds are the primary vertical pump: they draw in shallow water from a large region, and spit it out in a vertical fountain.
By default, this has minimal effect: it creates a bit of water churn.

With upgrades though, this can be tilted to the side, allowing you to move water up terraces.

## Converting water

TBD.
