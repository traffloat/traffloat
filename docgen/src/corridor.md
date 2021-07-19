# Corridor
Corridors connect terminals
to other terminals or other buildings.

## Appearance
Corridors are cylinders that connect
the centers of the two endpoint buildings.

- In the Normal Perspective,
coordiros are rendered with
the color of gases in the corridor.
- In the Electricity Grid Perspective,
each electricity grid gets assigned a color.
Each active power cable is rendered with
the color of its grid.

## Transportation
Corridors allow electricity, liquids, gases,
vehicles and inhabitants
to move across the endpoints.

When a new corridor is constructed,
gases can diffuse through,
and inhabitants can move across it freely.
Cables, pipes and rails can be added
by drawing circles on the cross-section view of the corridor.

### Electricity
Cables can be added to corridors
to connect the electricity grids of its two endpoints.
The cable can be manually disconnected
to split power grids.

Transferring electricity through cables consumes electricity.
Therefore, it may be useful to turn off
electricity cables that are not necessary.

### Gases
Gases diffuse through the corridor.
Fans can be installed on terminals to adjust the flow rate of gases.
The empty cross-section area of the corridor also affects
the rate of gas diffusion.
There is no restriction in the variety and pressure of gases.

### Liquids
Pipes can be constructed in corridors.
Each pipe can only transfer one type of liquid in one direction.
The throughput is computed based on the terminal pumps
and the viscosity of the liquid type being transferred.

### Vehicles
Rails can be built in corridors to allow vehicles to move.
The speed of the rail is bounded by the slowest truck in the rail.

### Inhabitants
Inhabitants can move across corridors freely.
