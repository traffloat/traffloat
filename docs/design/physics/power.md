# Power system

Power is the flow of electricity between sources (generators) and sinks (consumers).

Power consumption and transmission are orthogonal.
A reactor may consume power from multiple sources without connecting them into the same power network.

## Transmission

Power is transmitted through power conduits, a.k.a. "cables", across corridors.
A cable may be connected to:
- the corridor it belongs to
- a building at either end of the corridor
- a facility in the buildings at either end of the corridor
- another cable sharing an endpoint building

Buildings, corridors and facilities act as generators/consumers that are effectively connected to ground behind,
while cables are connected to other cables or generators/consumers transitively.

Example:

```
            gnd   gnd   gnd   gnd
             |     |     |     |
            cor   bdg   fac   fac
             |     |     |     |
             +-cbl-+-----+-----+
             |
            cbl
             |
 +-----+-----+-cbl-+-----+-----+
 |     |     |     |     |     |
cbl   cbl   cbl    |     |     |
 |     |     |     |     |     |
gen   gen   cor   bdg   fac   fac
 |     |     |     |     |     |
gnd   gnd   gnd   gnd   gnd   gnd
```

Cables, generators and consumers have a predetermined voltage rating.
A generator or consumer may only be connected to a cable equal to its voltage rating.

Consumers have a predetermined power rating.
Cables have a predetermind resistance rating based on material and shape,
and a current limit rating based on material.
The ratings are used for the following algorithm:

TODO

## Generators and consumers

There are three types of generators and consumers:

- All corridors, unless both edges are closed.
- All buildings.
- Facilities that are [reactors](reactor.md) with power input or output.

### Buildings as consumers

Buildings consume power from one of the following:

- a chosen cable in an adjacent corridor.
- a facility in the building that generates power.

The ruleset may restrict this to cables of specific voltages or generator facility types.

#### Power consumption

Building power consumption only includes the base [building maintenance costs](graph.md).
Facility consumption is separately considered.

#### Power deficit

Building power deficit has the following effects:

| Subsystem | Effect |
| :---: | :---: |
| [residents](resident.md) | serves as an ambient catalyst |
| [reactors](reactor.md) | at *x*% power, circuit breaker disconnects facilities (1-*x*)% of the time |
| [vehicle](vehicle.md) | at *x*% power, vehicles cannot perform inertial motion through the building |

Power deficit does not affect functioning of the building itself,
but it serves as a catalyst/condition for some facilities in the building.

### Corridors as consumers

Corridors consume power from a chosen cable in the corridor itself.
The ruleset may restrict this to a specific voltage.

#### Power consumption

Corridor power consumption is the sum of the following:

- base corridor operation cost (see [corridor maintenance costs](graph.md))
- powering vehicles on rails
- powering fans in conduits

#### Power deficit

Corridor power deficit has the following effects:

| Subsystem | Effect |
| :---: | :---: |
| [residents](resident.md) | serves as an ambient catalyst |
| [fluid](fluid.md) | conduit fans stop working |
| [power](power.md) | connections remain functional |
| [rails](vehicle.md) | all power-based vehicles lose propulsion |

### Facilities as generators/consumers

A reactor facility may specify cable ports to connect to,
with specified voltage ratings subject to the reactor type,
acting as reactor inputs and outputs.
A power deficit results in 

A kind of reactor is transformer,
which acts as a power consumer at one voltage and a power generator at another voltage.
This allows connecting consumers at one voltage to generators at another voltage
by dynamically scaling the consumption requirement.
Transformers would be demand-driven,
i.e. demand at the output network determines the consumption at the input network.
