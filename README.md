# traffloat

[![](https://github.com/traffloat/traffloat/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/traffloat/traffloat/actions/workflows/ci.yml)
[![](https://github.com/traffloat/traffloat/actions/workflows/client.yml/badge.svg?branch=master)](https://traffloat.github.io/master)
[![](https://github.com/traffloat/traffloat/actions/workflows/docs.yml/badge.svg?branch=master)](https://traffloat.github.io/api/master/traffloat)
[![](http://img.shields.io/badge/tech-stack-0690fa.svg?style=flat)](https://stackshare.io/sof3/traffloat)
[![GitHub](https://img.shields.io/github/last-commit/traffloat/traffloat)](https://github.com/traffloat/traffloat)
[![GitHub](https://img.shields.io/github/stars/traffloat/traffloat?style=social)](https://github.com/traffloat/traffloat)

A cooperative 3D web game with elements of City Building, Logistics and Tower Defense.

## Gameplay
The player manages a space colony.
Maintain the base, optimize logistics and defend from asteroid attacks.

The space colony is composed of buildings
connected by cylindrical corridors.
AI-controlled units transfer resources through the corridors
subject to different challenges.
Buildings produce or convert resources to enforce defense.

## Compilation
Logic is implemented in the `common` crate,
while code specific to rendering and user interface is located in the `client` crate.

The following tools are used for compiling the client:

- Ruby 2.7.0
- Python 3.8.10
- Node v14.16.0
- Rust (default toolchain) nightly-2021-06-28

Ruby is used to minify the GLSL scripts,
and Python is used for combining the texture images.
Node is used to run `webpack` and assemble the site.
Rust is used to write the main game logic compiled to WebAssembly.

To compile the client:

```shell
cd client
npm install
npm run preprocess
npm run build-dev
```

Change `build-dev` to `build` to create an optimized build.

The command `npm start` can be used to watch and recompile the project served on a dev server,
but it does not regenerate assets and minified GLSL code.

Create a thread in [Discussions](https://github.com/traffloat/traffloat/discussions)
if you would like to contribute and don't know where to start.
