# traffloat

[![](https://github.com/traffloat/traffloat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/traffloat/traffloat/actions/workflows/ci.yml)
[![](https://github.com/traffloat/traffloat/actions/workflows/client.yml/badge.svg?branch=master)](https://traffloat.github.io/master)
[![](https://github.com/traffloat/traffloat/actions/workflows/docs.yml/badge.svg?branch=master)](https://traffloat.github.io/api/master/traffloat)
[![](http://img.shields.io/badge/tech-stack-0690fa.svg?style=flat)](https://stackshare.io/sof3/traffloat)
[![GitHub](https://img.shields.io/github/last-commit/SOF3/serde-iter)](https://github.com/SOF3/serde-iter)
[![GitHub](https://img.shields.io/github/stars/SOF3/serde-iter?style=social)](https://github.com/SOF3/serde-iter)

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
