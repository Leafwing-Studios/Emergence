# *Emergence* within the Factory Builder Genre

Following the [genre analysis](genre-analysis.md), let's examine how *Emergence* fits within the factory builder genre.

## Game Thesis

With a biologically inspired setting and game mechanics,
*Emergence* attempts to fill a unique niche in a very succesful genre that is low on genuine innovation.

*Factorio*, a smash hit indie game, succeded because of its:

- relatively intuitive, easy to control UX
- satisfying, simple audio
- clear graphics
- rich and interesting contraption mechanics
- well-paced progression
- thriving modding community
- impressive performance
- intriguing fantasy: build a factory on an alien world and advance in technology

However, competitors in the space have often failed to catch audience's attention in the same way.
Getting the details right (particularly around UX, contraption mechanics, progression and performance) is remarkably hard.
Just as importantly, these competitors are by-and-large unimaginative, both in theme and mechanics.
Belts, inserters and machines in a generic sci-fi environment are all well-and-good,
but fail to capture the imagination of new audiences, or stand out from the crowd.

By taking the rich and fun core mechanics of factory builders,
adapting them to a unique and compelling theme,
and adding a unique mechanical focus on resilience,
*Emergence* looks to appeal to a loyal audience ravenous for something meaningfully new.

## Key Design Constraints

These are self-imposed, and chosen to create a more compelling and cohesive game:

- Players should be able to look at a screenshot or video of an inspiring contraption, and replicate that in their game.
- Colonies should always feel like bustling organic hives and farms, not mechanized factories.
- Optimal play should involve:
  - adapting to the local environment
  - automatically responding to predictable cycles
  - dealing with all waste products

## Mechanical Translation

### Core Mechanics

- **Resource patches**
  - Environmental resources are typically continuous, rather than discrete
    - Still clumped, but no clear boundaries
  - All resources are always renewable in some form
  - Soil contains various concentrations of nutrients (Nitrogen, Phosphorus, Potassium)
  - Soil is physically made up of sand/silt/clay/stone/organic matter
  - Water is vital for virtually everything, and can be found in streams, lakes, organic items and soil
  - Energy is similarly vital, but can be found in different forms that are edible to different organisms
    - Ultimately gathered via photosynthesis
- **Recipes**
  - Quite standard overall
  - Emphasis on byproducts and consequences of unmanaged waste
  - Circular processing
  - Items can decompose over time without action
- **Assemblers**
  - simple items can just be assembled by units at a crude shelf made of dirt (cheap) or stone (durable)
  - more complex items involve the use of dedicated plant or fungal buildings with selected recipes
- **Transporters**
  - ground units carry items and do work flexibly, but are physically grounded
  - flying units can travel more freely, but have lower carrying loads and much higher upkeep
  - water flows can be used to carry goods downriver
  - large quantities of goods can be transported via large, high-momentum ground units
- **Storage**
  - simple one-resource piles that are exposed to the elements
  - sheltered storage buildings made of plants and fungi
  - units and storage work together in a fashion directly analogous to Factorio's logistic network
- **Resource sinks**
  - lossy conversions: energy and water will commonly be lost during resource transformations
  - resource upkeep: energy and water will commonly be lost
  - pheromones: used to exert control over the colony and interactively boost effectiveness
  - guided evolution: research analogue used to modify and enhance existing species
  - assimilation: research analogue used to add new species to the colony
  
### Advanced Mechanics

- **Distributed resource costs**
  - water is used by everything, with rate varying by temperature and humidity
    - ultimately replenished by rainfall
    - distributed via:
      - canals
      - mycorrhizal networks
      - water droplets
  - energy is used by everything, with rate varying by amount of work done
    - ultimately gathered via photosynthesis
    - distributed by:
      - items
      - mycorrhizal networks
- **Fluids**
  - transported via plant and fungal networks
  - transported via canals
  - can be pumped
- **Filters**
  - units are capable of differentiation
  - simple mechanical filters sort items into classes
    - item size
    - floats
    - blows away
  - plants are capable of sophisticated filtration of solutions
- **Bypasses**
  - underground tunnels
  - overpasses
  - catapults?
  - bridges?
- **Spatial constraints**
  - rocks
  - trees
  - bodies of water
  - litter from extinct humans?
  - modifying the topology etc should be possible, but very expensive
- **Technology**
  - guided evolution: research analogue used to modify and enhance existing species
  - assimilation: research analogue used to add new species to the colony
- **Production enhancements**
  - pheromones: spent manually by the player to temporarily and locally enhance
  - upgrades: done via guided evolution, affecting all organisms of that strain
  - enhancements: pheromones can be produced and applied automatically via workers

### Supplementary Mechanics

These features supplement the core gameplay loop by providing additional things to do or consider, but are not needed.

- **Exploration**
  - important but not essential
  - fairly standard implementation
  - needs more interesting world generation
  - water distribution and topography are key
- **Combat**
  - natural but not essential
  - must be careful to avoid snowballing effects
    - design with negative feedback loops like hunger satiation
- **Pollution**
  - thematically essential
  - needs much richer (and more plausible) model of pollution
  - effects should be varied and depend on intensity of pollution
  - polluting should be easier than dealing with waste products properly
  - pollution can serve as yet-another important driver of temporal variation, as it builds up and must be dealt with

## Design Strengths

- biology serves as a great source of easy-to-explain inspiration for unique game mechanics
- compelling and unique thematics
  - opportunities for subtle political/moral storytelling on importance of sustainability and dangers of pollution
- disruptions offer a unique opportunity for players to explore more robust factory designs
- disruptions can create much more intresting emotional pacing in a genre that struggles with flatness
- tiny scale offers interesting mechanical and experiential possibilities that will feel new and interesting
- emphasis on sustainability pushes designers and players towards more interesting resource refinement pathways
- assimilating new species offers a natural and high impact path to adding more options for players
  - this can be done as horizontal progression, allowing new players to jump into whatever interests them most
- pheromones offer the player more ability to respond to the world (at a cost), and a more engaging gameplay experience
- lack of player avatar reduces frustration of inventory management and walking around
- modded Factorio has really nailed much of the UX and QoL features that we needed, and offers a clear base to learn from

## Design Challenges

- trap of realism
  - the availability of ecology and biology and ecology risks designers targeting realism over player experience
  - the world must be **plausible**, not realistic
  - for example, realistic evolution systems are likely to be fiddly and frustrating and distracting, rather than rich and interesting
  - some audience segments may be upset by "unrealistic" mechanics
    - mods are a helpful outlet for them
- scope creep due to abundance of fascinating ideas
  - game design must create reusable systems with excellent hooks
  - project management must focus on polishing and shipping complete systems
- design risk when testing new mechanics
  - needs agressive prototyping
- visual clarity is hard when allowing for player-driven species modifications
  - can we force each high-impact choice to modify exactly one element of the unit in a modular way?
- combat is a natural fit for the thematics, but is often clunky or frustrating in this genre
  - disembodied control helps
  - loose control over units (this is not an RTS) may be frustrating
- UX challenges when experimenting with new interaction paradigms
  - zoning
  - hex tiles
  - debugging unit behavior
  - more than 2D layouts?
  - tunnels?
  - information about soil composition?
  - information about signals?
- potential high performance costs of some mechanics
  - pathfinding
  - water flow
  - decomposition
- animation of units in a tile-based setting is an open question
- unit collision / interference is an open question
