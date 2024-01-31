# Structural graph

This crate provides base types and utilities
for the exterior structures in Traffloat.

There are two types of structures: buildings and corridors.
Buildings are connected together by corridors.

## Buildings

A building is a spherical structure.
Within a building, up to one "main facility" and multiple "storages" may be installed.

A main facility is the main functional part of a building.
The behavior of the facility determined by its "features",
e.g. [reactions](../reaction/README.md), [assignment](../assign/README.md),
[checkpoint](../security/README.md), etc.

A storage is a facility in which
[inhabitants](../inhab/REAMDE.md) and other resources may be located/stored,
including [electricity](../elec/README.md), [cargo](../cargo/README.md),
[fluids](../fluid/README.md) and [vehicles](../vehicle/README.md).

Each facility takes up space of a predefined shape
in the spherical wall of the building.
Players must select the position and orientation of a facility
during building construction/renovation
such that there is sufficient space.

The space in a building not ocucpied by any facilities
is known as the "ambient space",
which acts as a fluid storage and an inhabitant space.

## Corridors

A corridor is a cylindrical structure that connects two buildings.

Smaller cylindrical "ducts" may be installed within a corridor,
which transfer resources between endpoints.
Possible transferable resources include
[electricity](../elec/README.md), [fluids](../fluid/README.md)
and [vehicles](../vehicle/README.md).

Each duct takes up a circular area in the cross-section circle of the corridor.
Players must select the position of a duct
during corridor construction/renovation
such that there is sufficient space.

The area in a corridor not ocucpied by any facilities
is known as the "ambient duct",
which acts as a fluid storage and an inhabitant space.
