# Genre Analysis: Factory Builders

## Defining Games

- Factorio
  - colonize an alien planet and launch a rocket
- Satisfactory
  - open world first person factory building
- Shapez.io
  - abstract factory puzzles
- Dyson Sphere Program
  - gather resources from an entire solar system

## Defining characteristics

- automated resource extraction
- automation resource refinement
- logistical challenges
- micro-optimizations
- gated technology progression

## Adjacent Genres

- Base and city builders
  - Build out a complex and delightful area, but with no / little automation
- Zachlike puzzles (automation heavy puzzle games)
  - Micro-optimization as an entire game
- Colony sims
  - Balance complex systems, but with a focus on the needs of your workers, not scaling production
- Idle games
  - Automate everything, but without the complex chains
- Train simulators
  - Focusing almost entirely on logistic networks powered by trains
- Real Time Strategy
  - Command units and build bases and extract resources
  - Very little automation: resource production is almost entirely abstracted
  - Much more time pressure!

## Blended Genre Games

These games are all factory builders combined with the genre(s) in brackets.

- Mindustry (tower defense / RTS)
- Hydroneer | Astroneeer | Atrio (exploration)
- Automachef (Zachlike + cooking game)
- Infinifactory (Zachlike)
- Timberborn (city builder)
- The Incredible Machine (puzzle game)

## Progression Loop

- explore
- find resource patch
- exploit resource
- refine resource
- build into science
- unlock new technology
- optimize base with new technology
- explore to find new resource type to exploit

## Sources of Challenge

- player-driven optimization
- resource refinement puzzles
- logistics layout
- (optional) combat disrupting operations
- (optional) temporal variation creating instability in resource supply or demand

## Design Tensions

- complexity vs meaningful richness
- helpful innovations vs trivializing the game
- disruption forces interesting changes vs frustration and sadness to see your stuff get wrecked
- long play time vs tedious waiting around
- player fantasy of being in the game vs annoyance of inventory space and walking times
- giant automated bases vs computational limits

## Mechanics

### Core Mechanics

These are the basic building blocks needed to make the game fun.

- **Resource patches**
  - Extract raw resources from the environment from these
  - Sometimes limited, sometimes infinite
  - Commonly trees, ores and water
- **Recipes**
  - How different resources can be combined together
  - Raw resources become intermediates become end products (which have a genuine in-game use)
  - Recipes are very rarely reversible (or come at a high cost to do so), forcing players to consider which intermediate to transport
  - Example: iron ore (raw resource) can become iron ingots (intermediate) which are turned into gears (intermediate) which can be used to make belts (end product)
- **Assemblers**
  - Select a recipe for what you want to make
  - Not all assemblers can make all recipes
  - Can often be upgraded
  - Often paired with **inserters** in some form to load and unload resources
  - Commonly assembling machines, chemical plants, cooking stations or so on
- **Transporters**
  - Moves goods from place to place
  - Typically belts and pipes and trains and bots
    - Belts and pipes are efficient and good for small areas.
    - Trains move large amounts of goods in burst, but have a high investment cost
    - Bots are able to move goods in a more flexible and dynamic way, but require heavy upfront and ongoing costs
- **Storage**
  - Stores pools of resources in one place
  - Has a limited capacity
  - Commonly varies by size, cost to produce, footprint (space in the world) and materials that can be stored there
  - Mixed storage is usually possible, but almost always a noob trap
- **Resource sinks**
  - Provides ways to consume resources
  - Can be thought of as "the point"
  - Commonly: researching technology, combat, maintenance costs
  
### Advanced Mechanics

Game mechanics that are tightly integrated with the core loop and add rich complexity.
These are optional, but commonly included in some form.

- **Distributed resource costs**
  - Used to add a cost to actions
  - Must be transmitted through the base
  - Typically modelled as electricity
- **Fluids**
  - Requires a parallel distribution and storage network (in constrast to solid items)
- **Filters**
  - Splits mixed streams of goods
  - Belt splitters, liquid filters and inserters (via selective pickup) can server this purpose
- **Bypasses**
  - Underground belts and pipes
  - Allows more complex logistical configurations
  - Always more costly than alternative
- **Spatial constraints**
  - Features of the physical environment that must be worked around
  - Sometimes doubles as resource patches
  - Commonly cliffs, water or simply "end of map"
- **Technology**
  - process and spend resources to unlock new options
- **Production enhancements**
  - Modules: boosts the effectiveness of the building they are installed in
  - Beacons: boosts some factor of nearby buildings
  - Upgraded buildings: higher cost, but better throughput or efficiency
  - Researched passives: "everything of type X is now Y% more efficient"
  - Alternative recipe paths: more complex paths may be more efficient, or make use of alternative feedstocks

### Supplementary Mechanics

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
- **Pollution**
  - Created by extracting, refining and consuming resources
  - Waste products, atmospheric pollution
  - Discourages excess production
  - Can reduce productivity of other resources or provoke combat

### Quality of Life (QOL) Features

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
  - Often paired with a small, always-on-screen **minmap**
  - Augmented with **map markers**, which are player-made indicators of specific locations (ideally text + an icon)
- **Notes**
  - An in-game way to record what to do next, add flavor, or explain why something was done this way
  - Ideally tied to a location
  - Sometimes map makers are repurposed, sometimes this relies on in-game signs, or is spelled out manually using building mechanics

### Meta Features

These are optional ways to enhance the game experience and add replay value.
They do not live in the game itself.

- Tutorial
  - Learn to play the game via a simple, relatively scripted scenario.
  - Good UX design and achievements may be able to remove the need for this.
- Modding
  - Tweak the gameplay, tuning levers, aesthetics of the game
  - Add more content and systems
  - A natural fit for this genre!
- Small-group multiplayer
  - Play online with your friends
- Seeded world generation
  - Supply a fixed value for world generation, so you can play on the same map as others
- Map editor
  - Manually change the map
- Controllable world generation
  - Change the rules of the game (combat or not, pollution or not, resource costs) to customize play experience
  - Change the quantity and distribution of resource patches and spatial constraints

## Common Problems

- mediocre combat
  - poorly integrated
  - frustrating
  - snowbally
  - thematically incoherent
  - difficult to balance
- poorly managed complexity in UX
- poor tutorialization
- bland, unoriginal aesthetics
- unoriginal mechanisms
- extreme reliance on external guides
- boring and low-impact environmental variability
- copy-paste of optimized designs
- treadmill-style tech progression
- terrain modification that makes the world less interesting
- no meaningful penalties for overproduction of resources and mindless expansion

## Drivers of Player Churn

- overwhelming UI and bad control at start of game
- no clear goal at any point
- tedious tasks
- poor pacing
- performance issues

## Expected Business Model

- early access
- sandbox
- live service development
- (optional) modding
- (optional) small group multiplayer
- (optional) expansions
- NO costmetic microtransactions
- NO pay-to-win
