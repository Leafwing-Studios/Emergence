# Game thesis: *Emergence*

Following the [genre analysis](genre-analysis.md), let's examine how *Emergence* fits within the factory builder genre.

## Game Thesis

With a biologically inspired setting and game mechanics,
*Emergence* attempts to fill a unique niche in a very successful genre that is low on genuine innovation.

*Factorio*, a smash hit indie game, succeeded because of its:

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
  - Reflavored machinery (belts, gears, electronics) and human construction styles (houses, right angled construction, planks) don't cut it!
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

## Design Strengths

- biology serves as a great source of easy-to-explain inspiration for unique game mechanics
- compelling and unique thematics
  - opportunities for subtle political/moral storytelling on importance of sustainability and dangers of pollution
- disruptions offer a unique opportunity for players to explore more robust factory designs
- disruptions can create much more interesting emotional pacing in a genre that struggles with flatness
- tiny scale offers interesting mechanical and experiential possibilities that will feel new and interesting
- emphasis on sustainability pushes designers and players towards more interesting resource refinement pathways
- domesticating new species offers a natural and high impact path to adding more options for players
  - this can be done as horizontal progression, allowing new players to jump into whatever interests them most
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
  - needs aggressive prototyping
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
