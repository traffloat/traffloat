The project is structured into the following components:

- [physics](./physics/): Core simulation logic, including physics and resident simulation
- `client::scene`: Client-side code for receiving simulation updates and rendering
- `client::dock`: Client user interaction management
- server (TODO): A headless version that only runs physics and exposes them through network.

Conceptually, the world is split into two parts, the "physics world" and the "scene world".
The physics world is managed by `physics`,
which broadcasts updates to `client::scene` through the [protocol](./protocol/)
in the form of bevy message channels, potentially synced through the network.
`client::scene` builds a scene world that mirrors the physics world,
containing information of reduced detail/precision and injects rendering behavior.
