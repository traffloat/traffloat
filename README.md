# traffloat
A cooperative 2D web game with elements of City Building, Logistics and Tower Defense.

## Gameplay
You are managing a space colony.
Maintain the baae, thrive the economy and defend from asteroid attacks.

The space colony is composed of buildings
connected by cylindrical corridors.
Resources can be transferred among corridors.
The game starts with a Core building,
which is able to self-sustain the needs of a few workers
and defend from small asteroids.

### Electricity
Electricity is the fundamental resource for colony operations.
Electricity can be generated from Solar Cells placed at the peripherals of the colony.
Sunlight revolves along a fixed orbit,
so solar panels shall be constructed along the projection of this orbit.

### Gases
Gases can diffuse through buildings freely,
but gas pumps in corridors can speed up diffusion in a certain direction.
The normal air comprises different types of gases.
They can only be separated in special buildings like Fractional Distiller.

One type of gas is oxygen,
which is produced in oxygen farms and consumed by humans.
Workers would suffocate if oxygen level is too low.
In areas without oxygen, workers can also survive on oxygen packs delivered as cargo.

### Temperature
Buildings in the colony generate heat.
To avoid overheating, there must be sufficient space
among buildings and corridors for heat radiation.
Overheating would cause buildings to slow down or halt operation.

### Liquids
Liquids are transferred through pipes in corridors.
There may be a limited number of pipes in a corridor,
but each pipe only transfers one type of liquid in one direction.

Pipes can only be built if one of the endpoint buildings is a Terminal,
which provides the pumping force for the liquids.

### Cargo
Cargo can be carried by vehicles on magnetic rails in corridors.
Similar to liquids, the driving force is provided by Terminal buildings.
But unlike liquids, one rail may accomodate multiple vehicles,
but the speed is restricted by the slowest vehicle on the rail.

### Asteroids
Increasingly intense waves or asteroids strike the colony from fixed directions.
Defend the colon with buildings like Force Shield and Laser,
constructed on the direction from which asteroids spawn.

The Core is able to adjust the orientation of the whole colony
through massive electricity consumption.
This rotation is helpful in orienting the defense buildings against asteroids.

### Economy overview
To summarize the basic economy:

- Everything requires electricity
- Asteroids provide raw materials like Water and different ores.
- Water and electricity can be used to generate oxygen
	(through photosynthesis or electrolysis)
- Workers require oxygen to survive.
- Some buildings require workers to operate.
- Construction requires workers.
- Vehicle operation requires humans.
- Different buildings are constructed with ores.

## Contributing
Both the client and server are written in Rust.

To run the client from source, `cd` into client and use `npm start`,
which starts a develipment server.

To run the server from source, `cd` into server  and run `cargo run`.

Client-server communication uses WS(S),
so only TCP communication is used.
