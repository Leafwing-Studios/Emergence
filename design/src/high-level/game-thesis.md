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
- interesting engineering puzzles through well-designed production chains
- rich opportunities for optional optimization
- clear, well-paced progression
- thriving modding community
- impressive performance
- intriguing fantasy: build a factory on an alien world and advance in technology

However, competitors in the space have often failed to catch audience's attention in the same way.
Few have failed to capture the core gamplay loop to the same level of proficiency: they may have production chains,
but working them out is much more straightforward, and the tools provided for optional optimization are limited.
Additionally, getting the details right (particularly around UX, contraption mechanics, progression and performance) is remarkably hard.

Just as importantly, these competitors are by-and-large unimaginative, both in theme and mechanics.
Belts, inserters and machines in a generic sci-fi environment are all well-and-good,
but fail to capture the imagination of new audiences, or stand out from the crowd.
Even when working within the established genre, many still fail to provide exciting goals to work toward.
Power armor and cars and nuclear reactors are *cool*, and they keep players pushing forward.

By taking the rich and fun core mechanics of factory builders,
adapting them to a unique and compelling theme,
and adding a unique mechanical focus on resilience,
*Emergence* looks to appeal to a loyal audience ravenous for something meaningfully new.

## Key Design Constraints

These are self-imposed, and chosen to create a more compelling and cohesive game:

- Players should be able to look at a screenshot or video of an inspiring contraption, and replicate that in their game.
  - For a given factory design or contraption, all the moving parts should be visible and identifiable
  - It should be apparent, then, how each contraption works and how the resources "flow" through it
- Relatively light tone, playful art style
  - Solarpunk, not Zerg
- Colonies should always feel like bustling organic hives and farms, not mechanized factories.
  - The thematic focus z`should be on "nature reclaiming" instead of "industry civilizing"
  - An emphasis is put on the survival of the colony as a whole, not any individual member
- When designing your colony, robustness is valued over finely tuned optimization
  - Robustness to refactoring
  - Robustness to predictable environmental changes
  - Robustness to infrequent threats
  - Robustness to changes in supply and demand
- Optimal play should involve (but not require):
  - Adapting to the local environment
  - Dealing with all waste products
- Game mechanics are inspired by real world ecology, but making a clear and fun game with interesting choices comes first, even if that means bending reality somewhat

## Mechanical Translation

### Core Mechanics

- **Resource patches**
  - environmental resources are typically (but not always) continuous, rather than discrete
    - still clumped, but no clear boundaries
    - ex: nitrogen can be extracted from soil, but will be richer in some locations
  - all resources can be obtained renewably, with the right strategy
  - soil contains various concentrations of nutrients (nitrogen, phosphorus, potassium)
  - soil is physically made up of sand/silt/clay/stone/organic matter
  - water is vital for virtually everything, and can be found in rain, streams, lakes, organic items and soil
  - energy is similarly vital, but can be found in different forms that are edible to different organisms
    - ultimately gathered via photosynthesis
- **Recipes**
  - quite standard overall
  - emphasis on byproducts and consequences of unmanaged waste
  - circular processing
  - some items can decompose over time without action
- **Assemblers**
  - simple items can just be assembled by units at a crude shelf made of dirt (cheap) or stone (durable)
  - more complex items involve the use of dedicated plant or fungal buildings with selected recipes
- **Transporters**
  - ground units carry items and do work flexibly, but are physically grounded
  - water flows can be used to carry goods downriver
  - water flows can be expanded with player-dug canals
  - large quantities of goods can be transported via large, high-momentum ground units
- **Storage**
  - simple one-resource piles that are exposed to the elements
  - sheltered storage buildings made of stone, plants and fungi
  - shelted storage will reduce rate of decay, and mess due to rain and other effects
  - units, assemblers and storage work together in a fashion directly analogous to Factorio's logistic network
- **Resource sinks**
  - lossy conversions: energy and water will commonly be lost during resource transformations
  - resource upkeep: energy and water will commonly be lost to keep things alive
  - pheromones: used to exert control over the colony and interactively boost effectiveness
    - refine and then spend base resources to improve production rate
  - guided evolution: research analogue used to modify and enhance existing species
  - assimilation: research analogue used to add new species to the colony
  - hive mind: research analogue, used to unlock new features in the lab
  - ???: some kind of final goal to work towards building
  
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
  - units are capable of differentiating between items
  - some units won't care about some items via signal preference tuning
  - simple mechanical filters sort items into classes
    - item size
    - floats
    - blows away
  - plants are capable of sophisticated filtration of solutions
- **Splitters**
  - streams that split
- **Prioritizers**
  - nonlinear signal feedback loops?
  - stream geometry?
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
- **Multiple transportation options**
  - Many species, and species variants
- **Cyclic production pathways**
  - Core mechanic
- **Byproducts**
  - Core mechanic
- **Pollution**
  - thematically essential
  - needs much richer (and more plausible) model of pollution
  - effects should be varied and depend on intensity of pollution
  - polluting should be easier than dealing with waste products properly
  - pollution can serve as yet-another important driver of temporal variation, as it builds up and must be dealt with
  - several kinds:
    - solid waste
    - water pollution
    - soil pollution
- **Stochastic outputs**
  - Inherent in advanced tech
- **Degrading products**
  - Organic materials only
  - Most food
- **Hazardous goods**
  - Combat-focused items
  - Advanced tech
- **Environmental process bounds**
  - inherent to using living organisms as assemblers (and workers)

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
