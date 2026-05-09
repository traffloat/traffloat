# Power system

Power is the flow of electricity between sources (generators) and sinks (consumers).

Generators are [reactor](reactor.md) facilities with class `power::Generator`.
Consumers include all non-closed buildings, all non-closed corridors and any facility with class `power::Consumer`.

Power consumption and transmission are orthogonal.
A [fixture](graph.md) may consume power from multiple sources without connecting them into the same power network.

## Transmission

Power is transmitted through power conduits, a.k.a. "cables", across corridors.
Buildings can be configured to connect any pair of cables connected to the building.

Cables are undirected.
All generators and consumers in the same connected component of the power network
contribute to the total power generation and consumption of the component.

## Generation

Each facility with class `power::Generator` serves as a power source
for a specific network it is configured to connect to.
Such connection is defined as the network of a selected power cable connected to the building.

## Consumption

Each building and its facilities with class `power::Consumer` receive power from
any cable in any corridor docked with the building.
If there is a `power::Generator` facility in a building,
all facilities in the building are automatically powered
since they must be able to consume from the cable targeted by the generator.
An exception is when the building has a generator
but the generator did not select any target cables,
in which case the generator can still power the building itself as well as its facilities, effectively a local cable.

Corridors are powered by the any networks connected to either of its endpoint buildings.
but do not transmit power from them unless there is a cable in the corridor connected to them.
This means that a corridor can be powered by a building even without any connected cables.

If a consumer is connected to multiple power networks,
it automatically chooses the network with higher absolute power surplus (or lower absolute power deficit) to consume from.
However it is not possible for a consumer to consume from multiple networks simultaneously.

## Wrapping up producer/consumer graphs

Put differently, a power network is constructed by:

1. the set of cables connected to each other by explicit pairwise connections in buildings.
2. the set of buildings adjacent to cables in this set
3. the set of consumer facilities in these buildings
4. the set of corridors adjacent to these buildings

By corollary,

- a consumer can participate as a member of multiple isolated power networks
- a generator can only contribute to one isolated power network (but can consume from multiple if it requires starting power)
- a corridor can be powered by a cable of an adjacent corridor without having a cable connected to it.

## Storage

Facilities with class `power::Storage` can store power as a `u64` quantity.

A power storage behaves like a power consumer.
It receives excess power from any network its building is connected to
when the network has a power surplus and it has a capacity surplus.
Multiple storages in a network receive excess power simultaneously evenly.

A power storage also behaves like a power generator.
Similar to `power::Generator`s, it can be configured to connect to the network of a selected power cable.
It only generates power when the connected network would otherwise have a power deficit.

By corollary, a power storage consuming from multiple networks would
compensate the power deficit of its generation target network
with the power surplus of its consumption source networks,
but would not do the other way round.
This effectively serves as a unidirectional diode between the two networks.

## Power deficit

When power generation is less than consumption,
consumers stop receiving power in an order determined by a player-assigned priority.
Lower-priority consumers are shut down before higher-priority consumers.
The impact of lack of power depends on the type of consumer:

- If a building is unpowered, all facilities in the building are automatically shut down.
  Thus, a building effectively has higher priority than all of its facilities.
- If a corridor is unpowered, it is automatically [closed](graph.md#corridor-closure).
- If all endpoint buildings of a corridor are unpowered, the corridor is also unpowered.
- If a [reactor](reactor.md) facility is unpowered, it stops producing.
<!-- TODO impact on resident housing? -->
