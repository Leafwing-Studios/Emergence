# Glossary

These definitions are intended to be relatively brief and opinionated.
They are not authoritative!

<!-- Please try to keep the terms sorted alphabetically. -->

## Game Design

Key concepts and terms from game design.

### Accessibility

### Automation

### Challenge

### Design-by-stumble

### Depth and complexity

### Fun

### Genre

### Interesting choices

### Loop

### Mechanic

### Progression

### Tuning levers

### Tutorialization

## Factory Builders

Terms defined here are standard elements of the factory builder genre.

### Belt

A belt is a simple logistic entity which transports goods in a single direction.
Multiple belts need to be chained together to provide effective transportation.
Belts usually have a maximum amount of goods they can transport per unit of time.
They can transport any type of solid goods and different goods can be mixed.

**Examples**:

- [Factorio: Transport belt](https://wiki.factorio.com/Transport_belt)
- [Satisfactory: Conveyor belt](https://satisfactory.fandom.com/wiki/Conveyor_Belt)
- [Dyson Sphere Program: Conveyor belt](https://dyson-sphere-program.fandom.com/wiki/Conveyor_Belt)

### Bot

Bots are units which can automate specific player actions.
They often require a lot more upfront investments than machines, but are a lot more flexible.
Compared to machines, bots can move around freely.
Some actions can only be automated through bots, e.g. constructing new entities.

**Examples**:

- [Factorio: Construction robot](https://wiki.factorio.com/Construction_robot)
- [Factorio: Logistic robot](https://wiki.factorio.com/Logistic_robot)

### Blueprint

Blueprints are placeholders for multiple entities to be constructed.
They can serve as a planning tool for the player, but also be used as instructions for bots.
Often, blueprints can be shared between players and/or be reused during the game.

**Examples**:

- [Factorio: Blueprint](https://wiki.factorio.com/Blueprint)
- [Factorio: Ghost](https://wiki.factorio.com/Ghost)
- [Satisfactory: Blueprint designer](https://satisfactory.fandom.com/wiki/Blueprint_Designer)

### Clipboard

Similar to blueprints, the clipboard allows the quick reuse of factory layouts.
A set of entities can be marked to be copied and then pasted at a different position at a later time.
Usually, the entities are not built directly, but only pasted as placeholders.

In contrast to blueprints, players can usually only have one layout in the clipboard.
Previously copied layouts are simply overwritten.

**Examples**:

- [Factorio: Controls/Tools](https://wiki.factorio.com/Controls#Tools) (copy/paste/cut)
- [Dyson Sphere Program: Mass Construction (Lv1)](<https://dyson-sphere-program.fandom.com/wiki/Mass_Construction_(Lv1)>) (enables copy/paste functionality)

### Filter

Filters allow only one set of goods to pass through.
Can be combined with _Inserters_ to only transfer one set of goods
or with _Splitters_ to split one set of goods to one side and all other goods to the other side.

**Examples**:

- [Factorio: Filter inserter](https://wiki.factorio.com/Filter_inserter)
- [Factorio: Splitter](https://wiki.factorio.com/Splitter) (provides filter functionality)
- [Satisfactory: Smart splitter](https://satisfactory.fandom.com/wiki/Smart_Splitter) (provides filter functionality)
- [Dyson Sphere Program: Sorter](https://dyson-sphere-program.fandom.com/wiki/Sorter) (provides filter functionality)

### Inserter

Inserters transfer goods from one entity to another.
Often, inserters are required to insert goods into machines.
Inserters are usually available with different ranges, either as separate entities or as configuration option.

- [Factorio: Inserters](https://wiki.factorio.com/Inserters)
- [Dyson Sphere Program: Sorter](https://dyson-sphere-program.fandom.com/wiki/Sorter) (provides inserter functionality)

### Lab

A machine to research new technologies, unlocking new machines and other gameplay features.
Usually uses _science packs_ as input.

- [Factorio: Lab](https://wiki.factorio.com/Lab)
- [Satisfactory: HUB terminal](https://satisfactory.fandom.com/wiki/The_HUB#HUB_Terminal)
- [Dyson Sphere Program: Matrix Lab](https://dyson-sphere-program.fandom.com/wiki/Matrix_Lab)

### Logistic network

A network of connected logistic elements that can interact with each other.
It's composed of _producers_, who provide resources to the network, _consumers_, who take resources out of the network and use them, and the connection between them.
A single entity might be both a producer and a consumer.
In the same game there may be different separated sets of logistic networks.
The consumers in the network (e.g. logistic bots) can access the producers in the network (e.g. logistic chests), but not outside of it.

**Examples**:

- [Factorio: Logistic network](https://wiki.factorio.com/Logistic_network)

### Overlay

Overlays are UI elements which allow the player to view additional information about entities or processes.
They help convey information that is not easily visible by the in-game graphics alone.
The player can switch between different overlays to access different types of information or disable them entirely.

**Examples**:

- Factorio: "Alt mode"
- [Oxygen Not Included: Overlays](https://oxygennotincluded.fandom.com/wiki/Overlays)

### Pipette

The pipette is a tool to select an entity and/or its configuration directly from the screen.
It works similarly to copy/paste functionality with the _clipboard_, but for a singular entity.
This is usually a UI tool instead of a physical tool to build in the game.

### Power

Power is a special type of resource that is required for machines to run.
Power needs to be produced and then distributed to the machines.
Often, there are different means of production (e.g. steam power, solar power, nuclear power) with different upfront costs and production rates.
The distribution of power is usually separated from the distribution of other resources
(e.g. through an electric network).

- [Factorio: Electric system](https://wiki.factorio.com/Electric_system)
- [Satisfactory: Power](https://satisfactory.fandom.com/wiki/Power)
- [Dyson Sphere Program: Power](https://dyson-sphere-program.fandom.com/wiki/Category:Power)

### Research

Research is the main means of progression.
Researching new technologies unlocks more machines, enables more advanced processes or improves existing features.
Research is usually very expensive to pace the game.

**Examples**:

- [Factorio: Research](https://wiki.factorio.com/Research)
- [Satisfactory: Milestones](https://satisfactory.fandom.com/wiki/Milestones)
- [Dyson Sphere Program: Research](https://dyson-sphere-program.fandom.com/wiki/Research)

### Resource

An item that can be processed further to produce different items.

There are different types of resources:

- **Raw resource**: A foundational resource that can be extracted directly from the environment.
- **Intermediate resource**: A resource that cannot be extracted nor used directly.
  It's an intermediate set towards the production of end products.
- **End product**: A resource that can be used directly for some in-game benefit.

### Science pack

An intermediate resource that can be used to research technologies in the _lab_.
Often, many science packs are required to unlock one technology.
Sometimes, science packs are available in different types with different recipes,
different technologies require different science pack types to be unlocked.

**Examples**:

- [Factorio: Science pack](https://wiki.factorio.com/Science_pack)
- [Dyson Sphere Program: Science matrices](https://dyson-sphere-program.fandom.com/wiki/Category:Science_Matrices)

### Train

An entity to efficiently move big amounts of resources over large distances.
Usually require a much higher upfront investment than _belts_, but are better suited for long distances.
They are high latency, and move goods in clumped bursts.
In some games there are also many other entities to guide the behavior of large train networks.

**Examples**:

- [Factorio: Railway](https://wiki.factorio.com/Railway)
- [Satisfactory: Electric Locomotive](https://satisfactory.fandom.com/wiki/Electric_Locomotive)

## Ecology

These terms come from the science of ecology and its related fields.

### Clay

### Nutrient cycle

### Sand

### Silt

### Soil

### Water cycle

## Emergence

These terms are specific to the game design of _Emergence_.
Even if they have an external meaning, when used in this book, they will reflect this meaning.
