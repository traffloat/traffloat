# Vehicle system

Vehicles are individual entities that can carry residents and cargo over rails.

## Types

There are different types of vehicles as defined by mods.

Each type of vehicle has the following properties:

- Physical properties
  - Mass
  - Volume
  - Length
  - Gauge size
- Motion properties
  - Power source
  - Maximum speed
  - Maximum accelerating force
  - Maximum braking force
  - Drag coefficient
- Compartments
  - Passenger capacity
  - Fluid/cargo volume capacity
  - Isolation
- Operator slots
- Special classes
  - Construction vehicle, for constructing corridors and conduits
  - Tow vehicle, for pushing stuck vehicles off rails.

There are also separate rail types, which have the following properties:

- Gauge size
- Electrification
- Maximum speed

A vehicle type is compatible with a rail type if:

- The rail type is electrified, or the vehicle does not use rail power as its power source.
- The gauge size of the rail is equal to that of the vehicle.

In the setting of this game, rails are actually a pair of cylindrical rods
acting as prismatic joints to guide the vehicles as well as provide power if electrified.
The gauge size is basically the distance between the two rods,
which also effectively determines the diameter of the vehicle.
Mods may define narrow gauges for small vehicles and wide gauges for heavy vehicles for realism.

## Compartments

A vehicle has multiple isolated compartments (typically at least two, one for driving/cargo and one for fuel).

Each compartment has a fluid storage, which is connected to the ambient space by default.
Compartments with isolation properties may be disconnected from the ambient space,
allowing it to store volatile cargo without contaminating along its path.

Each compartment also has a specified limit of residents that can fit in.

## Power source

Vehicles have three possible power sources:

- Rail power: The vehicle consumes [electric power](power.md) from the rails,
  which are the power networks that the corridor can consume from.
- Fluid fuel: The vehicle specifies a compartment and fluid type for fuel storage.
- Cargo fuel: The vehicle specifies a compartment and cargo type for fuel storage.

Power/fuel is consumed only when the vehicle is accelerating or maintaining speed on a rail.
Both are defined by the rail type, where speed maintenance is basically a quantity relative to drag.

## Operator slots

The vehicle type defines a number of operator slots.

A vehicle cannot move if not all slots tagged as "driver" are occupied.
Operator slots may have a minimum attribute requirement
to ensure the vehicle can only be operated by residents trained for its operation.

Functional vehicles may have additional operator slots for non-driving purposes, e.g.:

- loadmaster for special freight vehicles
- technician for construction/maintenance vehicles
- fighter for combat vehicles

## Movement

One type of [conduit](graph.md) is "rail", which allows vehicles to move along the corridor.
At any instant, a vehicle is located either in a building ambient space or on a rail.

There are five rules of motion:
building-building, building-rail, rail-rail, rail-building,
and a special "inertial" rule for rapid rail switching across a junction building.

### Rule A: Intra-building non-inertial movement

When a vehicle is at rest in a building,
it can accelerate to the [standard walking speed](resident.md) immediately without using any fuel.

### Rule B: Building-to-rail non-inertial movement

A vehicle in a building can enter a rail when all of the following conditions are met:

- It is located near the point that the corridor docks with the building.
- The rail does not have any vehicles in the opposite direction.
- The first (initial length + safety distance) distance of the rail is clear of vehicles.

After entering the rail, the vehicle has zero initial speed.

### Rule C: Intra-rail movement

A vehicle on rail can select a speed `v` to accelerate to,
where `v` must satisfy all of the following conditions:

- `v` is less than the maximum speed of the vehicle.
- `v` is less than the maximum speed of the rail.
- There are no vehicles in the next `v^2 / (2 * b) + v * t + d` distance, where:
  - `b` is the maximum braking deceleration of the vehicle computed as force divided by mass
  - `t` is the reaction time buffer of the vehicle (constant 1 second)
  - `d` is the safety distance buffer between vehicles (constant)

The same mechanism is also used for deceleration.
After deciding on the new speed, the vehicles accelerates or decelerates based on the maximum allowed,
also computed as force divided by mass.
This leads to heavier or faster vehicles requiring more distance from the vehicle ahead of them.

### Rule D: Rail-to-building non-inertial movement

When the conditions for inertial movement (rule E) are not met,
a vehicle must reach less than or equal to standard walking speed to enter a building from a rail.

This means that a vehicle traveling at speed `v` must start braking
at `(v^2 - w^2) / (2 * b) + v * t` distance before reaching the end of the building,
where `w` is the standard walking speed,
and `t` is the buffer reaction time of the vehicle (constant 1 second).

### Rule E: Inertial movement for rapid rail switching

Consider a building `X` with two separate corridors `P` and `Q`.
A vehicle is traveling from an inbound rail in `P` at speed `v` towards `X`,
and plans to enter a rail in `Q` next.

The vehicle is eligible for inertial movement if
there exists a rail `q` in `Q` where all of the following conditions are met:

- `q` is compatible with the vehicle.
- The vehicle does not plan to stop at `X`.
- The vehicle plans to continue to `Q` after `X`.
- The initial `v^2 / (2 * b) + v * t + d` distance of `Q` is clear of vehicles,
  where `b`, `t` and `d` are defined similarly as above.
- `P` and `Q` form an obtuse angle at `X`.
- Facility volume occupancy in `X` is less than 40%.
- There are no other vehicles currently having reserved entry into `q`.
- There are no other vehicles currently having reserved segments intersecting with `PX-XQ` in `X`,
  except for those that start or end at `PX`.

> [!NOTE]
> In the 2D version, all corridor entrypoints are on the same plane, so it is very likely to intersect.
> In the 3D version, this intersection is instead the intersection of cylindrical volumes around the paths,
> where the cylindrical radius is a safety distance based on the gauge size.

Then the vehicle is eligible for transfer movement when it is within the braking decision distance for rule D:

1. The vehicle reserves the segment `PX-XQ` within `X`.
2. The vehicle reserves entry into `q`.
3. The vehicle accelerates or decelerates to an inertial speed `u <= max(w, v * -cos(PXQ))` before reaching boundary `PX`,
  where `w` is the standard walking speed, and `PXQ` is the interior angle between `P` and `Q` at `X`.
  - If `u = w`, the vehicle should prefer rule D instead since reservation only reduces efficiency.
4. The vehicle moves inertially through the distance from `PX` to `XQ` at speed `u`.
  During this period, the vehicle does not consume any fuel or power.
5. The vehicle enters `q` at speed `u`.
6. After the entire vehicle length is inside `q`, the vehicle releases the reservations for segment and entry,
  and switches back to rule C for intra-rail movement.

If the conditions are not met, rule D applies, requiring the vehicle to slow down to standard walking speed.
During the slowdown, eligibility for rule E may be re-evaluated and switch to rule E immediately when eligible.
In that case it may continue accelerating again to reach a higher `u` within bounds.

## Facility interaction

Vehicles may interact with [reactors](reactor.md) involving vehicle input
by moving near the center of a building and approaching less than standard walking speed.
This would despawn the vehicle and drop any cargo, fluid or resident in its compartments
into the ambient space of the current building.

Vehicles may be produced by reactors involving vehicle output,
which would spawn the vehicle at the center of the building at zero speed.
