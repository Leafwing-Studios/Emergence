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
- interesting, unexpected failures due to unexpected interactions or subtle bugs in the player's design
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
  - Greater focus on additional (seconday) optimization constraints (which are provided by the game)
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
  - Cookie Clicker, Candy Box
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
  - (insert roast of Factorio's logistics robots here)
- disruption forces interesting changes vs frustration and sadness to see your stuff get wrecked
- long play time vs tedious waiting around
- giant automated bases vs computational limits
- joy of optimizing and creating big factories vs uncomfortable implications of industrialization and colonialism
- accurate simulation of reality vs clear and interesting puzzle situations

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
- treadmill-style tech progression: same mechanics, but with a different coat of paint and "numbers go up"
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
