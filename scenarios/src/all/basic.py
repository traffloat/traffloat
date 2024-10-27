from dataclasses import dataclass
from typing import Self

from .. import assets, common_materials, cylinder, sphere
from ..glossary import Glossary
from ..save import fluid, Id, Writer
from ..save.building import Building
from ..save.facility import Facility
from ..save.fluid.container import Container as FluidContainer
from ..save.types import (
    TemplateDisplayText,
    Layer,
    Layers,
    PbrLayer,
    Position,
    Rotation,
    Scale,
)


def write_scenario(writer: Writer):
    glossary = Glossary(name="basic")
    fluids = Fluids.write(writer, glossary)
    ctx = Context(fluids=fluids, glossary=glossary)

    core(ctx, position=Position(x=-2.0, y=0.0, z=5.0)).write(writer)
    garden(ctx, position=Position(x=2.0, y=0.0, z=5.0)).write(writer)

    glossary.finalize(locales=["en"])
    writer.glossary_pool.push_finalized(glossary)


@dataclass
class Fluids:
    nitrogen: Id[fluid.Type]
    oxygen: Id[fluid.Type]
    co2: Id[fluid.Type]
    water: Id[fluid.Type]

    def write(writer: Writer, glossary: Glossary) -> Self:
        return Fluids(
            nitrogen=fluid.Type.gas_like(
                TemplateDisplayText.new(glossary, "Nitrogen"), 28.02
            ).write(writer),
            oxygen=fluid.Type.gas_like(
                TemplateDisplayText.new(glossary, "Oxygen"), 31.99
            ).write(writer),
            co2=fluid.Type.gas_like(
                TemplateDisplayText.new(glossary, "CO2"), 44.01
            ).write(writer),
            water=fluid.Type.aqueous(
                TemplateDisplayText.new(glossary, "Water"), 18.02
            ).write(writer),
        )

    def ambient_container(
        self, max_volume: float, max_pressure: float
    ) -> FluidContainer:
        """
        Standard atmospheric composition.
        """
        return FluidContainer(
            max_volume=max_volume,
            max_pressure=max_pressure,
            element_masses={
                self.nitrogen.id: 22400.0 / 14.0 * max_volume * 0.78,
                self.oxygen.id: 22400.0 / 16.0 * max_volume * 0.21,
                self.co2.id: 22400.0 / 16.0 * max_volume * 0.21,
            },
        )


@dataclass
class Context:
    fluids: Fluids
    glossary: Glossary


def core(ctx: Context, position: Position, rotation: Rotation = Rotation.identity()):
    return Building(
        position=position,
        rotation=rotation,
        scale=Scale.splat(2.0),
        label=TemplateDisplayText.new(ctx.glossary, "Core"),
        layers=Layers(
            distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            proximal=PbrLayer(
                mesh=sphere.Mesh(depth=5), material=common_materials.Glass()
            ),
            interior=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
        ),
        ambient_facility=Facility(
            fluid_containers=[
                ctx.fluids.ambient_container(max_volume=10000.0, max_pressure=100.0),
            ]
        ),
    )


def garden(ctx: Context, position: Position, rotation: Rotation = Rotation.identity()):
    return Building(
        position=position,
        rotation=rotation,
        label=TemplateDisplayText.new(ctx.glossary, "Garden"),
        layers=Layers(
            distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            proximal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            interior=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
        ),
        ambient_facility=Facility(
            fluid_containers=[
                ctx.fluids.ambient_container(max_volume=10000.0, max_pressure=100.0),
            ]
        ),
        other_facilities=[
            Facility(
                inner_scale=Scale(x=0.3, y=0.3, z=0.7),
                label=TemplateDisplayText.new(ctx.glossary, "Bushes"),
                layers=Layers(
                    distal=PbrLayer(
                        mesh=cylinder.Mesh(),
                        material=common_materials.RoughMonotone(r=0.39, g=0.85, b=0.34),
                    ),
                    proximal=PbrLayer(
                        mesh=cylinder.Mesh(),
                        material=common_materials.RoughMonotone(r=0.39, g=0.85, b=0.34),
                    ),
                ),
            ),
        ],
    )
