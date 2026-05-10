# Fluid system

Fluids operate by diffusion between fluid storages.

## Types

Fluids are managed as distinct, immiscible types.
Additional types may be defined by mods.

Fluids are stored as a `u64` molar quantities within each storage.
The sum of molar quantity for each type remains constant during transfer.
Only [reactors](reactor.md) may create or destroy molar quantity.

Each fluid type has the following properties:

- Specific heat capacity, used for conversion between heat energy and temperature
- Density, used for conversion between mass and molar quantity
- Advective viscosity, affecting the "drag" of moving fluids from container to container
- Diffusive viscosity, affecting diffusion rate
- Thermal conductivity, affecting heat conduction rate
- Wavelength spectrum, used for visualization and optical sensors

There is no concept of phase transition.
Liquids and gases would be immiscible types in a mixture,
but in this physics model they would behave as if the mixture has homogeneous physical properties in terms of transfer.

## Storage

There are several sources of fluid storages:

- Building ambient space, subject to the volume of vacant space in the building
- Corridor ambient space, subject to the area of vacant cross-section in the corridor
- Facilities with class `fluid::Storage`
- Conduits of type `Fluid`, a.k.a. "pipes"
- [Vehicle](vehicle.md) compartments

[Reactors](reactor.md) may be connected to other storages in the same building,
but they do not have their own storages.

## Connectivity

The connectivity between fluid storages depend on the source.

- By default, building ambient space is connected to the ambient space of all corridors docked with the building.
  - This connection may be closed with [corridor closure](graph.md#corridor-closure).
  - Base flow rate:
    - Area: the vacant area in the corridor cross-section.
      This is not customizable.
    - Resistance: length of the corridor.
- A storage facility may be optionally connected to fluid conduits of corridors docked to its building.
  - Base flow rate:
    - Area: the area of the conduit
      This may be arbitrarily reduced by the player.
    - Resistance: length of the corridor.
- A storage facility may be optionally connected to the ambient space of its building.
  - Base flow rate:
    - Area: the 2/3 power of the volume of the storage facility.
    - Resistance: the 1/3 power of the volume of the storage facility, or as defined by the facility.
- A vehicle compartment may be connected to the ambient space of the building or corridor it is in.
  - Base flow rate:
    - Area: as defined by the vehicle type
    - Resistance: as defined by the vehicle type

Fans may be installed at each connection point to alter the flow rate directionally.

## Transfer

A fluid mixture in each storage has the following source-of-truth attributes:

- Molar quantity: stored as a `u64` quantity
- Volume: determined by its storage
- Heat energy: stored as a `u64` quantity

Further attributes are derived from the above:

- Mass: as a product of molar quantity and density
- Temperature: as a ratio of heat energy to heat capacity
  - We exclude internal energy and only considers the *transferable* heat.
- Pressure: directly proportional to temperature and molar quantity, inversely proportional to volume

At each completed simulation frame,
we assume that each fluid mixture in each storage is homogeneous,
having thermal equilibrium and density equilibrium within the storage.

The transfer of fluids between each pair of connected storages
is the independent weighted sum of diffusion and advection.
The net movement of each fluid type is the sum of diffusion and advection.

### Advection

Advection is the movement of fluid mass due to the net directional flow.

1. Net force: (pressure difference \* area + fan force) \* [field affinity](field.md)
2. Net advection rate: net force \* delta time / resistance
3. Molar typed advection rate: net advection rate \* molar concentration / typed advective viscosity
  - where molar concentration is the proportion of the fluid type in the mixture in terms of molar quantity
4. Heat advection rate: sum(molar typed advection rate \* molar-specific heat capacity) \* source temperature

### Diffusion/convection

Diffusion is the mixing of fluid mass due to concentration difference regardless of net flow direction.
Heat convection similarly, exchanging heat energy due to mutual diffusion.

1. Base diffusion rate: area / length \* delta time
2. Molar typed diffusion rate: base diffusion rate / typed diffusive viscosity \* concentration gradient
  - where concentration of a fluid type in a mixture is the molar quantity divided by the volume of its storage
3. Typed heat convection rate: base diffusion rate / typed diffusive viscosity \* specific heat capacity
  \* temperature gradient \* sum of concentrations

### Conduction

Heat conduction is similar to diffusion, but is not affected by viscosity.

1. Base conduction rate: area / length \* delta time \* temperature gradient
2. Heat conduction rate: base conduction rate \* sum(concentration \* conductivity | for each fluid type)

## Computational caveats

Fluid molar quantity is stored as integers.
The list of fluid types in each storage is tracked as a sorted list of `(FluidType, u64)` tuples.
Only nonzero molar quantities are retained over frames.

Maximum net movement is clamped by half of the available molar quantity,
divided by the number of connections to each storage,
and rounded down to the nearest integer.

Note that backflow is still possible even if advection exceeds diffusion.
Furthermore, if a force equilibrium is reached between the fan and the pressure difference,
diffusion would still occur without advection,
which still results in diffusion equalizing the composition of the two storages.

Heat energy is transferred by three channels: advection, convection and conduction.
Convection behaves like diffusion and is directly proportional to the rate of convection,
while conduction is independent of fluid movement and depends on conductivity rather than heat capacity.

Computationally, all molar/heat transfer rates are computed in parallel by connection.
Results are grouped by storage, then transfer requests are applied in parallel by storage.

If a connection drains all distributed molar quantity of a fluid type from one side of a storage,
this observation would be tracked in the storage.
If it occurs consecutively for 3 frames, and there are no other connections on the same storage
with a net flow of this fluid type into this storage,
the fluid type would be fully transferred to the other side and removed from this storage.
If this occurs with multiple connections on the same storage, an arbitrary connection is selected.

This behavior would be negligible to the player since
rounding error is much less than advective flow rate,
and advective flow rate is much less than any quantity that causes observable changes to game mechanics.
