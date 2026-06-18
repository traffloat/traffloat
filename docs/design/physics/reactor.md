# Reactor system

Facilities with the `reactor::Reactor` facility class are the generic resource conversion facilities.
They can perform the following conversions:

- [Power](power.md): as a consumer or generator.
  The latter depends on additional configuration in the `power::Generator` class.
- [Cargo](cargo.md): consumes or produces cargo from/to a player-specified storage.
  May be a facility in the same building or ambient space of the building.
- [Fluid](fluid.md): consumes or produces fluid from/to a player-specified storage.
  May be a facility in the same building, ambient space of the building, or a fluid conduit in a connected corridor.
- [Resident](resident.md): increases or decreases resident attributes.
- [Vehicle](vehicle.md): one vehicle in the building can be despawned (consumed) or spawned (produced) at a time.

Furthermore, reactors can be *catalyzed* by certain conditions,
increasing or decreasing the reactor efficiency by its presence without being consumed.

- Temperature: the temperature of the [ambient fluid](fluid.md) in the building
- [Cargo](cargo.md): the amount of a cargo type in a player-specified storage
- [Fluid](fluid.md): the amount of a fluid type in a player-specified storage, and its temperature
- [Resident](resident.md): an attribute of a resident in an interaction slot of the facility
- [Fields](field.md): magnitude of a field
