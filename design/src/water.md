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

Let's focus on a few particularly relevant examples.

### Factorio

Water is a resource. It can be extracted endlessly and rapidly, and shipped huge distances.
It's vital for power production, but in base Factorio it's only used after that for oil production.

Water is also an obstacle, both for the player's factory and the biters. It serves as both a natural defense and limit to factory growth.
This can be overcome by building landfill, a mid-game tech that allows players to replace water with land.

This defense vs expansion tradeoff is by far the most interesting tension in how Factorio uses water.
Players want to build beside water to reduce the cost of defending a base (and have water for steam for power production),
but this will shape how their factory must grow.

Ultimately this doesn't end up mattering a ton, as marginal costs to longer distance transportation of goods is quite low.

Critically though, the utter impassability of Factorio's water severely hamstrings their map design.
Only lakes can exist, as river networks would both completely stop the ability for players to ship goods, explore or be attacked by biters.
Islands would be simply unreachable.

This means that interesting potential mechanics (choke points! river shipping! ocean liners! islands with unique resources!) are unexplored,
and water ends up feeling largely cosmetic.

### Sea Block

The AngelBob's mod pack for Factorio, especially in its SeaBlock form, do much more interesting things with water:

- as a thematic element: surrounded by water, you must find a way to make something from nothing
- as a source for hydrogen and oxygen
- as a source of raw resources, by purifying or evaporating water
  - mud, geodes, slag
- as a waste output from processes
- as a way to eliminate excess resources via clarifiers
- as an input for renewable plant growth (which then feeds animals or can be used directly to make key goods or power)
- as a solvent for chemicals like sulfuric acid
- to add serious costs (but not spatial constraints) to expanding the factory footprint

### Timberborn

*Timberborn* has by far the most sophisticated water dynamics in the genre to date.
Key features:

- seasonal variability
- steady evaporation
- controls where and when crops can be grown
- essential to sustain life
- flows downstream, and dynamically responds to height changes
- can be stored, either in open pits, via dams or in dedicated storage structures
- irrigation
- some buildings and crops require shallow water
- bridges of limited length to cross water
- limits navigation and complicates logistics

However, it has several key limitations:

- serious aesthetic issues: dried terrain looks horrible
- interesting water engineering mechanics are limited to late game
- drought mechanics are frustrating: limited ability to respond but strong ability to predict, leaving you slowly watching as your population dies
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
Water also has a temperature, and water's role as a heat exchange fluid

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
- be required by all plants to grow
  - roots are the primary mechanism
  - manually watering also works though
- be able to be meaningfully observed and understood by the player
  - weather and season cues
  - selection details
  - overlays
  - surface water visualization
- be able to be meaningfully manipulated by the player, especially in the mid and late game
  - terraforming allows players to durably shape the landscape, at fairly high time investment
  - as water flows downhill, players must be able to raise
- can be stored
  - surface-area based evaporation mechanics mean that deep holes are effective natural storage
  - lower loss storage should be possible, but expensive
- can be transported
  - canals work great for both irrigation and transport
  - more flexible (upstream, dynamic) mechanisms should be possible but expensive
- reach a stable equilibrium, even as water is added or removed from the system
  - surface-area based evaporation mechanics do an excellent job stabilizing this
  - draining to the ocean / filling up from the ocean also stabilizes this effectively
- be able to carry goods
  - goods should naturally float downriver (implying a water velocity)
  - goods should also be able to be ferried up river or across relatively still bodies of water at higher cost

Water should:

- play other meaningful roles in factory production chains
  - solution and evaporation of solids
  - to create mud
  - cooking
  - circular processing?
- be a useful trigger for conditional effects that players can use to respond to changes
  - seeds that only germinate in water / plants that only become non-dormant when wet
  - goods that begin to float and move when submerged
  - specialized storage that allows stored goods to float when submerged
- be fairly expensive to transport from place to place (to preserve spatial variability)
- be fairly expensive to store for long periods of time (to preserve temporal variability)
- flow downriver even over shallow gradients
  - this creates much more natural river designs
  - implies continuous water height

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
- fill up during the rain
- dry out over time (faster from deepr pools)
- be able to support crops on rainfall alone
- cause loose goods to float

Water should:

- create waterfalls
- create oceans
- create tidepools
- support tides
- leak out of imperfect holding vessels
- vary by biome (either causal direction is fine)
- run off the surface
- meaningfully interact with soil type in plausible ways
- come in different flavors
- carry dissolved / suspended solids
- cause organisms to drown
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

### Evaporation

Evaporation is simple: water is removed from each tile.
This varies with:

- the presence or absence of surface water
- the soil type (if no surface water is present)
- the light level on each tile (which in turn varies with local conditions, time of day and weather)

Because surface water evaporates at a much faster rate than soil water, this leads to a substantially stable equilibrium with rivers and islands.
As the amount of water increases locally, the rate of evaporation also increases automatically, creating a local balance.

### Precipitation

Precipitation is similarly simple: on each tile, add water based on the current weather.
This operates to refill water reserves that are far from rivers and oceans.

### Lateral water movement

Water flows from high to low.
The rate at which this flow occurs is proportional to the difference in height of the water column.

The overall effect creates a **flow velocity**, which can be used to transport floating goods, push units and more.

### Floating litter

Litter which is sufficiently light floats on the surface of the water, travelling in a rate (and direction) proportional to the flow velocity.
Each tile may only have one litter pile on it: litter that exceeds the stack size of the item (or is of a different type) piles up.

This effect is slightly randomized to reduce log jams and create a more visually appealing effect.
