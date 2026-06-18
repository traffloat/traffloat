# Core simulation

Traffloat is the result of inter-related integration of multiple core systems:

- The [graph](graph.md) system defines the structural layout of fixtures in the world.
- The [fluid](fluid.md) system simulates diffusion-based liquid and gas flow between different parts of the world.
- The [power](power.md) system simulates transmission of power between buildings.
- The [cargo](cargo.md) system provides stationary items that can be carried as granular bundles.
- The [resident](resident.md) system simulates colony population behavior,
  such as survival, psychology, jobs and decision-making.
- The [vehicle](vehicle.md) system simulates vehicles moving between buildings.
- The [field](field.md) system simulates environmental factors spanning the world
  independently of graph fixtures, providing quasi-terrain 4X experience.
- The [reactor](reactor.md) system provides facilities for all rule-based resource conversions.

## A note on dimensionality

The original goal of this game is to make everything truly 3D without ground,
but it turns out to be very mind-boggling for both development and gameplay to present and interpret a 3D world.

Fortunately, most core systems are more-or-less dimensionality-agnostic.
The initial design will be implemented as 2D,
with elements that are intrinsically dimensionality-dependent to be tracked actively, including:

- 2D graph must be a planar graph, while 3D graph requires other artificial restrictions.
  This affects construction validation.
- 2D buildings dock with corridors on a circle perimeter while 3D buildings dock on a sphere surface.
  This affects movement of residents and vehicles within buildings, especially during transfer.
- Directional fields interact with objects based on their dot product.
  This can be trivially solved by changing 2D dot product to 3D dot product.
- Inertial movement of vehicles involve 3D intersection testing,
  which involves different rules from 2D since it is much less likely to intersect in 3D space.
