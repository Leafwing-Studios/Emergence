# _Emergence_

[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/leafwing-studios/emergence#license)
[![CI](https://github.com/bevyengine/bevy/workflows/CI/badge.svg)](https://github.com/leafwing-studios/emergence/actions)
[![Discord](https://img.shields.io/discord/1027393534627692645?label=Discord&logo=Discord&style=plastic)](https://discord.gg/GyTG5KT352)

An organic factory-builder set in a micro-scale, post-apocalyptic future.
As the hive mind behind a growing, swarming, multi-species colony, you must learn to cultivate the world around you, adapting and literally evolving to face the hardships of a changing, alien world.

## About the game

The design documents are stored in the [`design`](./design) directory, and are created using [`mdbook`](https://rust-lang.github.io/mdBook/index.html).
Once you have `mdbook` installed, use `mdbook build --open` from the project root to read it in your browser!

For now, here are our high-level plans:

- build up a sprawling, complex factory-like colony that gathers resources from the environment, transports them through a complex supply network, and transforms them into outputs in a flexible fashion
- explore a procedurally generated 2.5D world that changes dramatically over both time and space
- create a self-sustaining, robust ecosystem that can adapt to even the most severe disruptions
- manage externalities, both positive and negative, and see the real effect of your actions on the water, soil and nutrients in your environment via rich information overlays and helpful charts
- experiment with unique forms of indirect control: nudge rather than command
- domesticate and integrate new species that you encounter in the world, each with unique gameplay effects
- shape the path of evolution, speciating and mutating your organisms to specialize and empower them for the jobs you need done

## Contributing

While we're looking to one day release and sell this game, the code is open source\*! As such, contributions are welcome.
If this game appeals to you (or you just want to learn more about making real games in Bevy), feel free to chip in and learn!
While the Serious Work happens here on Github, the dev team (and curious fans) hang out on the [Emergence Discord](https://discord.gg/GyTG5KT352).

Creativity and initiative is welcome:

- if you have a great idea for a new system, open a [Discussion] about it
- if you see some code that could be faster, cleaner or better, just open a PR

We follow an "optimistic merging" policy here: if the code is an improvement, and aligned with our goals for the game, we'll merge it in.
We're happy to help teach you, but for questions that don't relate specifically to _Emergence_ you should probably ask in the [Bevy Discord](https://discord.com/invite/bevy) (feel free to ping `@bzm3r`).

Ultimately, as maintainers, Leafwing Studios will maintain veto power over creative and technical decisions to help create a cohesive and compelling whole.
Monetization strategy is TBD still, but payouts will be split between donations to upstream projects and contributors (both on and off the Leafwing Studios team) who have invested significant time, money and expertise into the project.

Standard best practices from within the [Bevy ecosystem](https://github.com/bevyengine/bevy/blob/main/CONTRIBUTING.md) apply. Follow the code of conduct, write good issues, feel free to adopt PRs, and leave the code better than you found it!

\*see [Mindustry](https://mindustrygame.github.io/) for a great example of this working out!

### Getting started

1. Make sure you have [Git](https://git-scm.com/), [Git LFS](https://git-lfs.github.com/) and [Rust with Cargo](https://www.rust-lang.org/tools/install) installed.

2. Create a fork of this repository and clone it.

3. Pull the assets from the large file storage (LFS):

   ```cli
   git lfs pull
   ```

4. Run the project:

   ```cli
   cargo run
   ```

5. You can now make your changes on a new branch and open a pull request once you are ready!

## License

Emergence is free, open source and permissively licensed with the Rust ecosystem's standard dual MIT/Apache license.
Please don't rip off our game (unless it's dead, then fork it), but feel free to learn from it, upstream the clever bits and steal aggressively to make your own awesome games.

Non-font assets under `emergence_game` are currently licensed under CC-0. Commissioned work will be licensed under CC-BY-SA-NC (asset flippers suck).

Like usual, any contributions made are accepted under the same license terms. If you would like to modify or use these assets in your game, please reach out and we'll be happy to chat.
