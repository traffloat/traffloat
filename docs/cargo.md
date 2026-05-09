# Cargo system

Cargo is the concept of a stationary resource that can be transported in arbitrary amounts.

Cargo is measured as a `u64` quantity in terms of volume.

The game defines distinct cargo types with the following properties:

- Mass density correlating mass and volume
- Volatility, explained below

## Storage

Cargo may be stored in different storage locations,
which have various capacity properties limiting the total volume of cargo that can be stored inside.

- Cargo may be stored in [fluid](fluid.md) storages, reducing the volume of the fluid storage by the volume of the cargo.
  - Building ambient space
  - Corridor ambient space
  - Fluid storage facilities
  - [Vehicles](vehicle.md):
- Cargo may be carried by [residents](resident.md).
  The volume and mass capacity is determined by the strength of the resident.

## Volatility

Some cargo types are volatile, liberating or absorbing fluids or heat to the ambient space they are stored in:

- Fluid storages: operates on the fluid mixture directly. Heat energy is added to the whole mixture directly.
- Residents: volatility applies on the ambient space of the building, corridor or vehicle that the resident is located in.

## Transfer

Cargo can only be transferred between the following pairs of interactions:

- Resident and resident/ambient space/vehicle/facility:
  a resident can load or unload any amount of cargo to/from their inventory
  to another resident or storage, provided that the amount does not exceed the resident(s)' carrying capacity.
  This takes a fixed period of time, during which the resident cannot perform any other action.
- Vehicle and ambient space/facility: a vehicle can transfer cargo to/from a storage
  when they are located in the same building and the vehicle is stationary.
  The rate of transfer is determined by the vehicle's propreties.
