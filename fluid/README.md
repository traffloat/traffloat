# Fluid systems

This crate implements the fluid simulation logic in Traffloat.

The fluid model is a simplified model of real-world physics.
For simplicity, both liquids and gases are generalized as fluids.

## Storage

Fluids are stored in fluid storages ("containers"),
which are either building storages, corridor ducts,
[ambient space](../graph/README.md)
or [ambient duct](../graph/README.md).

Each container contains a mixture of immiscible fluids
corresponding to different fluid types.
The amount of each fluid in the mixture is represented by its mass.

### Mixture phases

The volume of a mixture measures the space it occupies within the container,
The volume is limited by the container size.
The terms "density" and "specific volume" refer to the ratios
`mass / volume` and `volume / mass` respectively.

The pressure of a mixture measures the force exerted by the fluid on the container
(thus, the pressure is never negative even if it is vacuum).
The pressure is limited by the container material.

The formula for volume and pressure depends its phase:

#### Vacuum phase

A mixture is in the vacuum phase if `sum(mass[type] / vacuum_density[type]) < volume_limit`,
where `vacuum_density` is a property of each fluid type.

During the vacuum phase, the volume is computed as `sum(mass[type] / vacuum_density[type])`.
The pressure is `sum(mass[type]) / volume_limit`.

> The real-life equivalent of "vacuum density" is approximately
> the density of a fluid under weightlessness held together by surface tension.

#### Compression phase

A mixture is in the compression phase if
`sum(mass[type] / vacuum_density[type]) > volume_limit`.

During the compression phase, the volume is always `volume_limit`.
The pressure is computed as `pressure = sum(mass[type]) / volume`.

> This is a very rough approximation of the ideal gass law `PV=nRT`,
> assuming constant molar mass, constant temperature
> and ideal gas properties during compression stage.

#### Saturation phase

For a single type of fluid,
it transitions from the compression phase to the saturation phase
when its pressure exceeds the `critical_pressure` defined for the fluid type.

At the saturation phase, each additional mass amplifies the pressure by `gamma` times.
The value of `gamma` is defined as a constant `65536.0`.
Thus, the volume continues to be `volume_limit`,
but the pressure is instead computed as
`mass / volume * gamma + critical_pressure * (1 - gamma)`.

For mixtures, the formula above is computed by adding up different pressures, i.e.

```text
pressure = sum(
  mass[type] / volume * gamma
  + critical_pressure[type] * (1 - gamma) * mass[type] / sum(mass)
)
```

> This is a hack to simulate liquids and fluids with low compressibility,
> allowing a container to be fully filled without hitting the pressure limit easily.
> The extra mass added during saturation phase
> is an indirect indication of the amount of pressure exerted by external forces.

#### Explosion phase

A container explodes if its pressure exceeds the pressure limit
for two consecutive simulation frames.
All connections are cut, and an `ExplosionEvent` is emitted,
the impact of which is to be handled by other modules, for example:

- The [construction](../construction/README.md) module
  may apply effects on the building attributes.
- The [inhab](../inhab/README.md) module
  may apply effects on inhabitants adjacent to the container.

## Transferring fluids

The fluid model avoids creating or destroying fluid mass.
Fluid may move between containers,
but the total mass is generally consistent.

Due to rounding error or computation efficiency,
a small amount of mass may be lost/created during simulation,
but this amount is generally intended to be kept negligible.

### Transfer links

Fluids may transfer between two containers within the same building
or between a container in a building and a duct in an adjacent corridor.
Such pairs are called "container links",
which may be modified through construction/renovation.

### Transfer rate

Fluid transfer across a pipe is computed from four variables:
resistance, pressure, force and diffusion.
To generalize, the transfer rate of a fluid type across a link is computed by

```text
pressure_gradient / resistance + force + concentration_gradient
```

#### Resistance

Resistance is the cost of moving a fluid from one link of a storage to another.

For corridor links, the resistance is a simplified version
of the flow rate multiplier given by the Hagen–Poiseuille equation,
i.e. `resistance = length / radius^4`, where `length` and `radius` describe the duct.

For inter-building links, we assume the length is the radius of the building itself,
and assume the whole building space is available for transfer,
so the resistance is simply `1 / radius^3`.

#### Force

The force is the sum of active transport effects exerted by other modules, for example:

- A pump installed on the link.
- A [field](../fluid/README.md) that affects flow rate.
