# traffloat
[![GitHub Pages](https://img.shields.io/github/workflow/status/SOF3/traffloat/Pages)](https://sof3.github.io/traffloat/master/)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/SOF3/traffloat/CI)](https://sof3.github.io/traffloat/master/doc/traffloat/)
[![StackShare](http://img.shields.io/badge/tech-stack-0690fa.svg?style=flat)](https://stackshare.io/sof3/traffloat)


A cooperative 3D web game with elements of City Building, Logistics and Tower Defense.

## Gameplay
You are managing a space colony.
Maintain the base, thrive the economy and defend from asteroid attacks.

The space colony is composed of buildings
connected by cylindrical corridors.
AI-controlled units transfer resources through the corridors
with realistic traffic rules.
Buildings produce or convert resources to enforce defense.

## Contributing
This game is written in Rust and compiled into WebAssembly.
Logic is implemented in the `common` crate,
while code specific to rendering and user interface is located in the `client` crate.

To run the client from source, `cd` into client and use `npm start`,
which compiles the project and starts a develipment server.

Create a thread in [Discussions](https://github.com/SOF3/traffloat/discussions)
if you would like to contribute and don't know where to start.
