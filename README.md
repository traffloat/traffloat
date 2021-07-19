# Traffloat

[![](https://github.com/traffloat/traffloat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/traffloat/traffloat/actions/workflows/ci.yml)
[![](https://github.com/traffloat/traffloat/actions/workflows/client.yml/badge.svg?branch=master)](https://traffloat.github.io/master)
[![](https://github.com/traffloat/traffloat/actions/workflows/docs.yml/badge.svg?branch=master)](https://traffloat.github.io/api/master/traffloat)
[![](http://img.shields.io/badge/tech-stack-0690fa.svg?style=flat)](https://stackshare.io/sof3/traffloat)
[![GitHub](https://img.shields.io/github/last-commit/traffloat/traffloat)](https://github.com/traffloat/traffloat)
[![GitHub](https://img.shields.io/github/stars/traffloat/traffloat?style=social)](https://github.com/traffloat/traffloat)

A 3D traffic, factory, city building, tower defense web game.

[Play the game](https://traffloat.github.io/master/) \|
[User guide](https://traffloat.github.io/guide/master/) \|
[Rust API docs](https://traffloat.github.io/api/master/) \|
[Discuss](https://github.com/traffloat/traffloat/discussions) \|
[Report bug](https://github.com/traffloat/traffloat/issues)

## What is this game about?
This game happens in a self-sustaining space colony.
The player constructs buildings and corridors in the colony
to produce and transfer different resources.

This game is about **logistics**.
Most resources are created from other types with factories.
Insufifciency or excess of resources will bottleneck the game.

This game is about **traffic**.
Resources are transferred in different forms (cargo, liquid, gas, electricity, human).
Optimize transportation routes to maximize production rate.

This game is about **city building**.
Inhabitants are produced in the colony to boost productivity.
Build schools to improve inhabitant skills and unlock new technologies.

This game is **3D**.
Explore new transportation mechanisms in a
[genuinely 3D][tvtropes-2d-space] network of buildings.
Become a leading architect for 3D cities.

  [tvtropes-2d-space]: https://tvtropes.org/pmwiki/pmwiki.php/Main/TwoDSpace

This game is **cooperative**.
Players can join the colonies of other players,
or create new colonies and establish trades with existing colonies.

This game involves **tower defense**.
Colonies are attacked by waves of asteroids in the space,
which can be propelled or dissolved into raw resources.

## Compilation
The following tools are used for compiling the client:

- Ruby 2.7.0
- Python 3.8.10
- Node v14.16.0
- Rust (default toolchain) nightly-2021-06-28

Ruby is used to minify the GLSL scripts,
and Python is used for combining the texture images.
Node is used to run `webpack` and assemble the site.
Rust is used to write the main game logic compiled to WebAssembly.

See the [justfile](justfile) for common commands, in particular:

```shell
# Install other dependencies
just deps
# Preprocess assets
just pp # Compile the client and start a dev server, and recompile if files changed just watch
# Compile the client for production
just build
```

While development build claims to produce "optimized" output,
LTO (link time optimization) is enabled in release build
to further improve performance and reduce file size
at the cost of longer compile time.

## Contribution
The game is composed of multiple crates:

- [`codegen`](./codegen)/[`codegen-types`](./codegen-types): Defines procedural macros used in the game.
- [`units`](./units): Defines standard data types of units used in the game.
- [`vanilla`](./vanilla): Defines vanilla game configuration.
- [`common`](./common): Implements game world simulation.
- [`client`](./client): Implements a web game client.
- [`docgen`](./docgen): Generates the markdown used to generate <https://traffloat.github.io/guide/master/>.

Create a thread in [Discussions](https://github.com/traffloat/traffloat/discussions)
if you would like to contribute and don't know where to start.
