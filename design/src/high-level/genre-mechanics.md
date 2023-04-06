# Factory-Builder Mechanics

## Core Mechanics

These are the basic building blocks needed to satisfy the core gameplay loop of the genre.

- **Resource patches**
  - Extract raw resources from the environment at specific locations
  - Sometimes limited, sometimes infinite
  - Commonly trees, ores and water
- **Recipes**
  - How different resources can be combined together
  - Raw resources become intermediates become end products (which have a genuine in-game use)
  - Recipes are very rarely reversible (or come at a high cost to do so), forcing players to consider which intermediate to transport
  - Example: iron ore (raw resource) can become iron plates (intermediate) which are turned into gears (intermediate) which are combined with iron plates to make belts (end product)
- **Assemblers**
  - Often, select a specific recipe for the assembler to make
    - Sometimes inferred from inputs
  - Not all assemblers can make all recipes
  - Can often be upgraded
  - Often paired with **inserters** in some form to load and unload resources
  - Commonly assembling machines, chemical plants, cooking stations or so on
- **Transporters**
  - Moves goods from place to place
  - Examples: belts, pipes, trains, bots
    - Belts and pipes are efficient and good for small areas
    - Trains move large amounts of goods in burst, but have a high investment cost
    - Bots are able to move goods in a more flexible and dynamic way, but require heavy upfront and ongoing costs
- **Storage**
  - Stores pools of resources in one place
  - Has a limited capacity
  - Commonly varies by size, cost to produce, spatial footprint, and materials that can be stored
  - Mixed storage is sometimes possible, but almost always a noob trap
- **Resource sinks**
  - Provides ways to consume resources
  - Can be thought of as "the point"
  - Commonly: researching technology, combat, maintenance costs
  
## Advanced Mechanics

Game mechanics that are tightly integrated with the core loop and add rich complexity.
These are optional, but commonly included in some form.

- **Distributed resource costs**
  - Used to add a cost to actions
  - Generally required by basically everything, but has much weaker spatial constraints for transportation and distribution
  - Must be transmitted through the base
  - Typically modelled as electricity
  - Often something that is revisited, scaled up, and upgraded through the course of a playthrough
- **Fluids**
  - Requires a parallel distribution and storage network (in contrast to solid items)
  - Ex: using pipes and tanks instead of belts and chests
- **Filters**
  - Splits mixed streams of goods
  - Belt splitters, liquid filters and inserters (via selective pickup) can serve this purpose
- **Splitters**
  - Divides a stream of goods into two or more parts, generally evenly
  - Ex: belt splitters, pipes
- **Prioritizers**
  - One use of a resource, either locally or globally, is deemed "more important" than others
  - May be able to prioritize both input and output!
  - Goods will be diverted to the more important path until that path is backed up
  - Ex: belt sideloading, splitter priority
- **Bypasses**
  - Underground belts and pipes, train intersections
  - Allows more complex logistical configurations
  - Always more costly than alternative
- **Spatial constraints**
  - Features of the physical environment that must be worked around
  - Sometimes doubles as resource patches
  - Commonly cliffs, water, finitely sized planets, or simply "end of map"
- **Technology**
  - process and spend resources to unlock new options
- **Production enhancements**
  - Modules: boosts the effectiveness of the building they are installed in
  - Beacons: boosts some factor of nearby buildings
  - Upgraded buildings: higher cost, but better throughput or efficiency
  - Researched passives: "everything of type X is now Y% more efficient"
  - Alternative recipe paths: more complex paths may be more efficient, or make use of alternative feedstocks
- **Multiple transportation options**
  - Multiple options for transporting goods that have distinct tradeoffs (setup cost, latency, throughput, batching)
- **Cyclic production pathways**
  - Some outputs must be processed and reused as inputs
  - Forces more interesting and more challenging factory designs
  - Examples: Angel's farms, Angel's slurry filters
- **Byproducts**
  - Some outputs of a factory process are undesirable
  - These must be reused for another process, recycled into something else, or desposed of at some cost
- **Pollution**
  - Created by extracting, refining and consuming resources
  - Have only seen atmospheric pollution
  - Discourages excess production
  - Can reduce productivity of other resources or provoke combat
- **Stochastic outputs**
  - Some factory processes don't always produce the same output, and instead produce one of several outpus randomly
  - Forces more robust designs, especially with regard to timing and surge capacity
- **Degrading products**
  - Goods that have a "shelf-life", and become less useful or turn into waste over time
  - Most commonly used for food
  - Often adds storage constraints
  - Often requires carefully managing throughput of production
- **Hazardous goods**
  - Goods that are dangerous to store, especially in excess
  - Punishes overproduction
  - Creates storage constraints
  - Examples: explosives, flammable goods, realistic electricity
- **Environmental process bounds**
  - Some steps can only be done when conditions are in the right range
  - Commonly seen in Oxygen Not Included: specific temperature ranges, atmospheric gases, pressure ranges

## Supplementary Mechanics

These features supplement the core gameplay loop by providing additional things to do or consider, but are not needed.

- **Exploration**
  - Fog of war
  - Maps
  - Additional zones to build and explore in
  - Usually but not always paired with a player avatar
- **Combat**
  - Adds another goal beyond research
  - Adds challenge and excitement
  - Often becomes more challenging as production grows (to avoid mindless exploitation)

## Quality of Life (QOL) Features

These things make the game loop more pleasant:

- **Cut-copy-paste**
  - Select groups of buildings (and their settings), and add them to your **clipboard**
  - Buildings that are cut are **marked for deletion**
  - Selections can be flipped and rotated
  - Paste these buildings to create **ghosts** (phantom buildings that are **marked for construction**)
  - Ghosts can then be built later by hand or via bots
- **Undo**
  - Reverse previous actions or directions
  - Best paired with **redo**
  - Redone actions will leave ghosts, rather than actually placing the tiles
- **Pipette**
  - Add a copy of buildings (and their settings) to your cursor
- **Blueprints**
  - Save copy-pasted designs
  - Share them with friends
- **Recipe look-up**
  - Figure out what items can be turned into
  - Figure out how items can be made
- **Research search**
  - Search for terms in the research tree, and see what is needed to unlock various recipes
- **Production statistics**
  - View how much of each resource you are producing and consuming over time
- **Alerts**
  - Warn the player when something that requires urgent action has occurred
- **Labs**
  - Prototype and measure designs in a sandbox environment
- **Production planner**
  - Analyze theoretical performance and ratios of resource pathways
- **Map**
  - Summarizes the area visually
  - Often paired with a small, always-on-screen **minimap**
  - Augmented with **map markers**, which are player-made indicators of specific locations (ideally text + an icon)
- **Notes**
  - TODO lists are a common and important use case
  - An in-game way to record what to do next, add flavor, or explain why something was done this way
  - Ideally tied to a location
  - Sometimes map makers are repurposed, sometimes this relies on in-game signs, or is spelled out manually using building mechanics
- **Overlays**
  - Display information about various important factors on the map or in-world display
  - Extremely useful to visually communicate the operation of various systems without cluttering aesthetics
- **Time control**
  - Pause, speed up or slow down time
  - Pausing and slowing down is useful for accessibility and to respond to crises
  - Speeding up is used to accelerate through boring parts of the game
    - This is probably a design smell that should be dealt with rather than papered over
- **State indicators**
  - See how machines are configured
  - See if machines are working
  - See what's inside storage
  - Usually toggled to reduce clutter

## Meta Features

These are optional ways to enhance the game experience and add replay value.
They do not live in the game itself.

- **Tutorial**
  - Learn to play the game via a simple, relatively scripted scenario.
  - Good UX design and achievements may be able to remove the need for this
- **Modding**
  - Tweak the gameplay, tuning levers, aesthetics of the game
  - Add more content and systems
  - Often a built-in manager for downloading and enabling mods
- **Small-group multiplayer**
  - Play online with your friends
- **Map editor**
  - Manually change the map
- **Controllable world generation**
  - Change the rules of the game (combat or not, pollution or not, resource costs) to customize play experience
  - Change the quantity and distribution of resource patches and spatial constraints
  - Provide a set seed so others can play the same map as you
- **Alternate terminal goals**
  - Achievements, score counters, etc.
  - Provides alternate metrics to optimize above simply creating the required products
- **Challenge scenarios**
  - Specific world or factory conditions that make the game harder
  - Ex: ribbon worlds, death worlds, seablock, missing resources, etc.
- **Social media sharing**
  - Easily share factory designs and entertaining moments with others
- **Wiki**
  - Online repository of information about the game
