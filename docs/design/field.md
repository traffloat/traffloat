# Field system

Fields are "terrain" features that exist independently of the graph system.
[Fixtures](graph.md) are affected by the strength of fields at their position,
providing enhancements or debuffs to their functions.

## Definition

Formally, a field is a function that maps a point in the world to a vector.
Depending on the effects of the field, the vector is either one-dimensional (scalar) or n-dimensional,
where n is the dimensionality of the graph positional system (2 or 3).

## Types

Mods can define types of fields in different aspects.

### Field effects

Examples:

- Efficiency of specific [reactors](reactor.md)
  - Example: solar panels are more efficient when there is a strong light field
- Acceleration, drag and braking efficiency of specific [vehicles](vehicle.md), conditionally directed
  - Example: vehicles brake less efficiently when there is a strong hypermoisture field
- Viscosity of specific [fluids](fluid.md), conditionally directed
  - Example: mercury becomes more viscous when there is a strong metallic field
- Base flow rate of fluid connections, conditionally directed
  - Example: fluids advect faster due to a gravitational field
- Specific attributes of [residents](resident.md) staying in the field
  - Example: residents lose health due to a strong radiation field

### Possible countermeasures

Examples:

- Construction of building/corridor upgrades, e.g. additional lead lining to block radiation fields
- Construction of facilities that radiate field modifiers, e.g. a heat radiator to enhance the exterior temperature field

### Terrain generation patterns

The game generates field magnitudes by Perlin noise.
Field types can apply transformations on Perlin noise output.

## Detection and research

Fields are mostly invisible and cannot be trivially detected.
A player initially notices the presence of a field from its effects on fixtures.

Laboratories can be bulit to research the mechanisms of the fields.
Research is more efficient if the player correctly identifies the field effect and areas it is stronger in.

After sufficient research is completed, new building/corridor upgrades and facility classes are unlocked.
Players must install sensor facilities across the colony to detect field strength or prospect unclaimed territories.

## Dynamic fields

When the dynamic field gamerule is enabled, new types of fields are randomly created over time,
based on the rules defined by the mods providing the possible enhancements and debuffs.
A randomly generated field places itself to avoid affecting existing fixtures,
but may affect future fixtures built out of the current territory
or newly constructed facilities

## Affinity

Type definitions have an affinity property.
Types with better affinity may be applied together in the same field when a new dynamic field is generated.
