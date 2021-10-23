# Traffloat

[![](https://github.com/traffloat/traffloat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/traffloat/traffloat/actions/workflows/ci.yml)
[![](https://github.com/traffloat/traffloat/actions/workflows/client.yml/badge.svg?branch=master)](https://traffloat.github.io/master)
[![](https://github.com/traffloat/traffloat/actions/workflows/docs.yml/badge.svg?branch=master)](https://traffloat.github.io/api/master/traffloat)
[![](http://img.shields.io/badge/tech-stack-0690fa.svg?style=flat)](https://stackshare.io/sof3/traffloat)
[![GitHub](https://img.shields.io/github/last-commit/traffloat/traffloat)](https://github.com/traffloat/traffloat)
[![GitHub](https://img.shields.io/github/stars/traffloat/traffloat?style=social)](https://github.com/traffloat/traffloat)

[![Simple issues](https://img.shields.io/github/issues/traffloat/traffloat/D:%20Simple)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22D%3A+Simple%22)
[![Medium issues](https://img.shields.io/github/issues/traffloat/traffloat/D:%20Medium)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22D%3A+Medium%22)
[![Complex issues](https://img.shields.io/github/issues/traffloat/traffloat/D:%20Complex)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22D%3A+Complex%22)

[![API issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20API)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+API%22)
[![Bug issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20Bug)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+Bug%22)
[![Documentation issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20Documentation)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+Documentation%22)
[![Feature issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20Feature)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+Feature%22)
[![Optimization issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20Optimization)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+Optimization%22)
[![Tooling issues](https://img.shields.io/github/issues/traffloat/traffloat/G:%20Tooling)](https://github.com/traffloat/traffloat/issues?q=is%3Aissue+is%3Aopen+label%3A%22G%3A+Tooling%22)

A 3D traffic, factory, city building, tower defense web game.

[Play the game](https://traffloat.github.io/master/) \|
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
This project uses Rust nightly-2021-10-09.
The toolchain version is automatically installed
if you installed Rust with `rustup`.

This project requites the wasm target for the Rust toolchain.
Install it by running the following command inside a traffloat clone:

```shell
rustup target add wasm32-unknown-unknown
```

This project uses [just](https://github.com/casey/just)
to manage script commands,
and [trunk](https://github.com/thedood/trunk)
to manage WebAssembly and site building.
To install these tools:

```shell
cargo install trunk just
```

See the [justfile](justfile) for common commands, in particular:

```shell
# Compile the scenario files
just client-scenarios-dev
# Compile the client for production
just client-build
```

To compile for development mode, use `just client-build-dev` instead.
There is also `just client-watch`,
which compiles the client and start a dev server,
and recompiles if files have been modified.

While development build claims to produce "optimized" output,
LTO (link time optimization) in release build
can further improve performance and reduce file size
at the cost of longer compile time.
The release build also triggers `wasm-opt`
with optimization level 4, which takes a long time to execute
(more than 15 minutes of CPU time).

## Contribution
The game is composed of multiple crates:

- [`codegen`](./codegen)/[`codegen-raw`](./codegen-raw): Defines procedural macros used in the game.
- [`types`](./types): Defines standard data types of vectors, units and gamerule definition.
- [`types`](./def): Defines scenario schema.
- [`common`](./common): Implements game world simulation.
- [`client`](./client): Implements a web game client.
- [`tfsave-builder`](./tfsave-builder): CLI tool to compile scenarios from human-readable TOML files.

Create a thread in [Discussions](https://github.com/traffloat/traffloat/discussions)
if you would like to contribute and don't know where to start.
