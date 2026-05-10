# Graph system

Rigid structure in the world comprises four types of fixtures:
buildings, corridors, facilities and conduits.
Buildings and corridors form an exterior structural graph.
Facilities and conduits are fixtures within buildings and corridors respectively.

Buildings are shaped as spheres.
Facilities are unshaped structures that take up the space of the building.

Corridors are shaped as cylindrical tubes with a circular cross-section.
Conduits of different sizes are presented as smaller circles along the cross-section.

In addition to facilities and conduits,
buildings and corridors also have ambient space, which is the interior space not occupied by any fixtures.
The ambient space serves as a [fluid](fluid.md) storage
as well as a space for [residents](resident.md) to move through.
[Vehicles](vehicle.md) also move across building ambient space either by inertia
or with the same (lower) speed as resident motion.

While the dimensionality between buildings is undetermined,
buildings and corridors are 3D internally.
In the simplified 2D camera, they are rendered as circles and rectangles instead,
with interior fixtures rendered as fixed-size squares/lines.

## Construction

### Corridor construction

Corridor construction must always start from a building.
The planned construction may end at nowhere or dock with another building.

Construction materials are loaded into a construction [vehicle](vehicle.md),
which travels to the construction site and starts building the corridor as required.
The construction vehicle moves with the end of the corridor completed;
it is not possible to use the corridor for any purpose until the planned construction completes.
After completion, a corridor with a single rail compatible with the construction vehicle type is created.

#### Costs

The construction cost of a corridor:

- scales quadratically with its length
- scales quadratically with its cross-section area (i.e. 4th power with its radius)
- plus a constant base cost

Corridors consume maintenance power.
This cost scales inversely with the squared distance between
its midpoint and the closest midpoint of other corridors or buildings in the world except the ones it connects with.
This cost deincentivizes building corridors that are too close to each other,
which would result in excessively packed layouts.

#### During construction

Until both ends of a corridor are docked with buildings,
its ambient space is unusable, and the only usable conduit is the rail for the construction vehicle,
only accessible by the construction vehicle itself.
If a corridor is undocked at both ends (due to building destruction),
the corridor itself is also destroyed in the same way as the building.

### Building construction

Building construction must always start at the end of an open corridor.

Construction materials are loaded into a construction [vehicle](vehicle.md),
which travels to the undocked end of an open corridor.
During the construction process, nothing except an internal progress value is updated,
but the constructed building may be partially rendered for visual feedback.

#### Costs

The construction cost of a building:

- scales quadratically with its volume (i.e. 6th power with its radius)
- plus a constant base cost

Buildings consume maintenance power.
This cost scales inversely with the squared distance between
its midpoint and the closest midpoint of other buildings in the world.
This cost deincentivizes building buildings that are too close to each other,
which would result in excessively packed layouts.

#### During construction

Until a building is completely constructed,
it is completely unusable and does not actually exist in the world.

### Facility construction

A facility must be constructed within a building.
Its position within the building is currently undetermined,
but they quantitatively consume building vacant space.
The amount of vacant space in the building affects storages in its ambient space
and the speed of vehicles transferring through it.

Construction materials are loaded into a construction [vehicle](vehicle.md),
which travels to the building and starts building the facility as required.

Demolishment or upgrading of a facility is treated as equivalent to construction.

#### During construction

During construction, most facilities in the building become unusable.
Specific examples of usable facilities include:
- [vehicle](vehicle.md) storage (but cannot be moved in or out except for the construction vehicle itself)
- [power](power.md) connections
- [fluid](fluid.md) connections, including both ambient space and fluid conduits

### Conduit construction

A conduit must be constructed within a corridor.
Its position within the corridor must be selected before construction starts.

Construction materials are loaded into a construction [vehicle](vehicle.md),
which moves along the corridor in either direction as the construction progresses.

#### During construction

During construction, all rails in the corridor become unusable except for the construction vehicle itself.
The ambient space is closed off.
Fluid and power conduits remain usable.

Demolishment or upgrading of a conduit is treated as equivalent to construction.

### Corridor closure

Corridors are closed off when they are under construction or have conduits under construction.
When a corridor is closed off,

- its ambient space must be vacated of all residents
- all [fluid](fluid.md) connections across the ambient space must be closed
- [fluid](fluid.md) connections to pipes are still functional, but fans stop operating.
- all [power](power.md) connections are still functional.
- the corridor stops consuming maintenance [power](power.md).
- all [rails](vehicle.md) in the corridor become unmoveable, except for non-power-based vehicles.
  An exception is active construction vehicles,
  which are only unusable if the corridor was forcibly closed due to power deficit.
  All vehicles currently on its rails will stop moving and emergency brake.
