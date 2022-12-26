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

- automated resource extraction and refinement
- logistical challenges driven by spatial constraints
- optional optimization of existing systems (generally optimizing resource throughput)
- gated technology progression
- problems/puzzles/modules flow into each other (as opposed to puzzle games or zachlikes where they are explicitly independent)
- partially self-directed goals
- challenge can be overcome by spending time waiting
- highly moddable
- procedurally generated world

## Why is this genre fun?

- puzzle like, but complete solutions are easy to achieve
- optional optimizations on top of said solutions
- satisfaction of watching your contraptions run
- satisfaction of taming the wild
- interesting, unexpected failures due to unexpected interactions or subtle bugs
- gameplay can be extended, enhanced, or changed through mods or built in options
- hits on "engineering creativity" in a way that other games don't
  - engineering creatvitiy is more about coming up with solutions within constraints. It's not a blank canvas. It's based on analytically evaluable metrics
- scratches similar itches to programming, but with clearer gradients and fewer frustrations
  - great debugging: can see what's working and why with exceptional visualization
  - basic solutions are straightforward
  - very weak syntax constraints, very intuitive syntax
  - solutions generally do not need to be as robust as in the real world
  - tech progression doesn't break previous (natural) assumptions
  - good quality of life tools and docs

## Adjacent Genres

This list is roughly ordered by the closeness to the genre.

- Zachlike puzzles (automation heavy puzzle games)
  - Opus Magnum, Incredible machine
  - Greater focus on additional (seconday) optimization constriants (which are provided by the game)
  - Micro-optimization as an entire game
- Base and city builders
  - Sim City, The Sims, Cities: Skylines, Dragon Quest Builders
  - Build out a complex and delightful area, but with no / little automation
  - Most of the actual functioning of your constructions is abstracted away
- Colony sims
  - Rimworld, Dwarf Fortress, Oxygen Not Included
  - Balance complex systems, but with a focus on the needs of your workers, not scaling production
  - More about the emergent push and pull of life in the colony. Less about steadily pushing towards some goal.
- Idle games
  - Forager, Cookie Clicker, Candy box
  - Automate everything, but without the complex chains
  - Additional mechanics and optimization opportunities unfold over time
- Train simulators
  - Open TTD, Mini Metro
  - Focusing almost entirely on logistic networks powered by trains
- Real Time Strategy
  - Starcraft, Warcraft III, Red Alert
  - Command units, build bases, and extract resources
  - Very little automation: resource production is almost entirely abstracted
  - Much more time pressure!
- Level builders
  - Y'know, like Mario Maker
  - Focused on creating interesting experiences and, in some cases, creating fun and interesting contraptions
- 5X games
  - Civ 5, Stellaris
  - A little city builder, a little RTS
- Tower defenses
  - Dungeon Warfare 2, Bloons TD, Gemcraft
  - In high quality games, there are strong optimization and contraption elements

## Blended Genre Games

These games are all factory builders combined with the genre(s) in brackets.

- Mindustry (tower defense, RTS)
- Hydroneer | Astroneeer | Atrio (exploration)
- Automachef (Zachlike, cooking game)
- Infinifactory (Zachlike)
- Timberborn (city builder)
- Minecraft (survival crafting / creative sandbox)

## Progression Loop

- acquire goal (generally something to produce)
- determine recipe
- (optional) unlock tech prerequisites
- find relevant inputs (raw resources or previously produced items)
- (optional) if necessary, rework existing parts of the factory to produce relevant inputs
- build prototype
- (optional) optimize and scale prototype
- integrate with larger factory
- reap fuits of your labor
- (ad hoc) discover and fix issues with previous designs

## Sources of Challenge

- player-driven optimization
- determining production chains
- physical layout of factory elements (making sure they reach each other and all fit, etc.)
- transport logistics (moving things around the factory)
- refactoring previously constructed parts of the factory as new requirements (or better methods) are discovered
- returning to your factory after a break, and needing to refresh yourself on how it works
- (optional) combat disrupting operations
- (optional) temporal variation disrupting operations, often by creating instability in resource supply or demand

## Design Tensions

What goals of the genre are in tension with each other?
Note that many of these design tensions are not unique to factory builders.

- mechanical complexity vs meaningful richness
- helpful innovations vs trivializing the game
  - unlocking new tech can sometimes remove interesting challenges intead of creating interesting opporutnity for improvement
  - (insert roast of factorio's logisitcs robots here)
- disruption forces interesting changes vs frustration and sadness to see your stuff get wrecked
- long play time vs tedious waiting around
- giant automated bases vs computational limits
- joy of optimizing and creating big factories vs uncomfortable implications of industrialization and colonialism
- accurate simulation of reality vs clear and interesting puzzle situations

## Mechanics

### Core Mechanics

These are the basic building blocks needed to make the game fun.

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
  
### Advanced Mechanics

Game mechanics that are tightly integrated with the core loop and add rich complexity.
These are optional, but commonly included in some form.

- **Distributed resource costs**
  - Used to add a cost to actions
  - Generally required by basically everything, but has much weaker spatial constraints for transportation and distribution
  - Must be transmitted through the base
  - Typically modelled as electricity
  - Often something that is revisited, scaled up, and upgraded through the course of a playthrough
- **Fluids**
  - Requires a parallel distribution and storage network (in constrast to solid items)
  - Ex: using pipes and tanks intead of belts and chests
- **Filters**
  - Splits mixed streams of goods
  - Belt splitters, liquid filters and inserters (via selective pickup) can serve this purpose
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
  - Often adds storage constriants
  - Often requires carefully managing throughput of production
- **Hazardous goods**
  - Goods that are dangerous to store, especially in excess
  - Punishes overproduction
  - Creates storage constraints
  - Examples: explosives, flammable goods, realistic electricity
- **Environmental process bounds**
  - Some steps can only be done when conditions are in the right range
  - Commonly seen in Oxygen Not Included: specific temperature ranges, atmospheric gases, pressure ranges

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
  - See what's instead stoage
  - Usually toggled to reduce clutter

### Meta Features

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

## Common Problems / Room for Improvement

- mediocre combat
  - poorly integrated
  - frustrating
  - snowbally
  - thematically incoherent
  - difficult to balance
  - limited depth
- poorly managed complexity in UX
- poor tutorialization
- bland, unoriginal aesthetics
- unoriginal contraption mechanisms
  - everyone uses inserters and belts
- boring and low-impact environmental variability
- copy-paste of optimized designs
- treadmill-style tech progression
  - driven by lack of end products
- terrain modification that makes the world less interesting
- no meaningful penalties for overproduction of resources and mindless expansion
- frustrating control schemes, especially in games with a player avatar
  - also results in difficulty seeing the whole factory at once
- missing QOL features

## Drivers of Player Churn

- overwhelming UI and bad control at start of game
- players are intimidated by engineering
- large portion of base is destroyed/disabled
- positive feedback loops on failure
- difficulty recovering from failure states
- no clear goal at any point
- tedious tasks
- poor pacing
- performance issues

## Expected Business Model

- early access
  - incorporates player feedback
- live service development
- (optional) modding
- (optional) small group multiplayer
- (optional) expansions
- NO microtransactions
