<!---
This document is a draft that will be gutted and used in the design docs later. 
--->

# Considerations for Representing the Game World

## How Things Work

### Terrain

Our current terrain and pathfinding systems work by defining normal ground, high ground, and rocky terrain, with the rocky terrain then labelled as impassible. However, most insects can climb vertical surfaces and most non-insect animals planned are far larger than insects and thus unencumbered by most terrain we're currently representing. This means no terrain should be impassible for our units and only certain terrain types should be "impassible" to fungi and plants given the substrate requirements either kingdom of life have to grow. We also need to consider how to represent the traversal of our terrain: will units walk across ground and respond to variations in it or will they simply know to go to certain tiles given the labels?

### Movement

Our current movement system has all the ants moving randomly across the map by spawning and de-spawning between tiles. We need to decide on how we wish to animate these movements. We will also need to consider how to process all of our unit actions. Current pathfinding is processed as time passes in the game with a unified system to process all units using pathfinding at once. For performance, we will need to consider how to distribute the processing load for large numbers of organisms pathfinding all at once.

## Planning for Development

### Basic Traversal

To make units moving around the world look more alive, we will want to add animations to smooth out character movement, decide how the units will move across different terrain, and how to represent the terrain. Our best solution for general pathfinding will be to rework our current impassible terrain functions and instead label the height of a tile. A height allows us to use the integer value to let organisms with walking mechanics know they cannot pass a tile/to know they will have to walk differently on this terrain as well as tell the renderer what terrain type to show.

We are planning to add new assets, a new lighting system, and a day/night cycle, which will all make height easier to represent. The most limiting factors for each choice of dimensions will then be the assets and extra work needed to finesse our game to be more performant and beautiful.

### Making the World 2D

Most factory builders tend to be 2D

    * We make units able to respond to height.
        - Pros:
          * We can generalize our movement system and adapt it to each organism as needed. 
          * If organisms have some level of awareness of their surroundings, most of the way that assets interact can be handled through this system with out needed finesse to pull off a good looking end result. 
          * It may be easy for us to add this to our current systems.
          * Signal transduction may be easier to work out
          * It's easier to work with
        - Cons: 
          * There are a lot of 2d factory sim type games, so looking unique will take a lot of thought
          * Will need to trouble shoot our assets for clear visibility 
          * Will likely require adding a physics engine (may also be a pro)
         
    * Smaller units look like they're getting larger as they climb up and get closer to the camera or appear to get slightly smaller as they climb down. Meanwhile, larger units are able to semi easily move across most terrain  
        - Pros:
          * Looks unique
          * May be easier to deal with animations ( we will only need the walking and smaller up/down sprites vs full animation set)
        - Cons:
          * Likely requires a query for each tile type (computationally intensive)
          * Less straight forward to implement than regular animation
     
    * Some mixture of the above 2 options
        - Pros: 
          * Could prove to be unique
          * Can adapt to any art style in unique ways
          * Flexibility in design offers developer ergonomics
        - Cons:
          * Could be more resource (time, computation, and/or asset) intensive 
          * Less straight forward to implement than either a physical world 

### Making the World 2.5D

There is no real consensus on what 2.5D means. This allows us to get extremely creative as to how we show the game world. We can:

     * Add 2D sprites to a 3D world, 
        - Pros
          * Can look interesting and add texture/personality to the game
          * Is easier to produce and manage assets for than full 3D
          * Makes some aspects of representation, such as depth and height, easier than fully 2D/3D
        - Cons
          * More labor and computationally expensive than just 2D
          * Adds complexity to the design/production process that may be less straight forward to deal with than fully either 2D/3D
     
    * Create a 3D world with a constrained camera,
        - Pros
          * You have all assets in 3D (render issues will be because of easier to troubleshoot 3D vs  potentially difficult mixed dimension)
          * Fits the genre well
          * Makes terrain easy to work with from a traversal perspective
        - Cons
          * You have all assets in 3D (you have to make, store, and deal with 3D assets meaning more time/money)
          * Added complexity
          * 

    * Mix 2D and 3D assets to our hearts' content (and coding/asset production ability). 
        - Pros 
          * Flexible and offers us the ability to use assets/techniques that are easiest for the task at hand
          * Is as unique as we can make it (More artistic freedom)
          * Allows us to accent/emphasize certain game world features that may help with UX and making the game easy to learn
          
        - Cons
          * Troubleshooting could get difficult
          * There will always be a need to troubleshoot rendering/ensuring assets are consistent across the board
          * Mostly uncharted territory, especially in bevy
          * Need to come up with a new design plan
          * May become difficult to pull off with more features/assets

### Making the World 3D

3D will make representation easier but adds layers of difficulty to design and production that may make it less practical than 2/2.5D.

    * 3D with isometric camera with freedom on the constrained axis
        - Pros
            *
        - Cons
            * 
     
    * Add ,
        - Pros
          * 
          * 
        - Cons
          * 
          * 

    * Mix 
        - Pros 
          *
        - Cons
          * 
