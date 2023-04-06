# Life Cycles

As organisms grows and responds to its environment, it will change between life forms.
A blade of grass may (in gameplay terms) begin as a seed, become a sprout, and then finally become an adult plant, which is capable of producing more seeds.
A dragonfly may begin as an egg, hatch into a larvae, become a nymph and finally become an adult before laying more eggs.
A tulip may begin as a bulb, become a sprout, flower, produce other seeds, and then return to its bulb form over winter.

As you can see by these examples, life cycles are a versatile tool for communicating important changes to players and can be quite elaborate, with multiple paths from a given state.

## Life cycle graphs

We can represent these changes via a state machine graph, where each node is a **life form**, and each edge is a **life path**.
Each path between forms is triggered by certain conditions: enough time, enough consumed food, cold temperatures, long light exposures, flowers were pollinated or so on.
