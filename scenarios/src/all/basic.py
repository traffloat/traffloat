from dataclasses import dataclass
from typing import Optional, Self

from .. import common_materials, cylinder, sphere
from ..glossary import Glossary
from ..save import fluid, Id, Writer
from ..save.building import Building
from ..save.corridor import Corridor
from ..save.duct import Duct
from ..save.facility import Facility
from ..save.fluid.container import Container as FluidContainer
from ..save.types import (
    Appearance,
    ConcatDisplayText,
    DisplayText,
    Layers,
    LiteralDisplayText,
    PbrLayer,
    Position,
    Rotation,
    Scale,
    TemplateDisplayText,
)


def write_scenario(writer: Writer):
    glossary = Glossary(name="basic")
    fluids = Fluids.write(writer, glossary)
    ctx = Context(fluids=fluids, glossary=glossary)

    core_id = core(ctx, position=Position(x=-2.0, y=0.0, z=5.0)).write(writer)
    garden_id = garden(ctx, position=Position(x=2.0, y=0.0, z=5.0)).write(writer)
    corridor(
        ctx,
        alpha=core_id,
        beta=garden_id,
        other_ducts=[
            fluid_duct(
                ctx,
                1,
                rgb=(0.8, 0.4, 0.7),
                inner=(0.0, 0.0),
                radius=0.05,
            ),
        ],
    ).write(writer)

    glossary.finalize(locales=["en"])
    writer.glossary_pool.push_finalized(glossary)


@dataclass
class Fluids:
    nitrogen: Id[fluid.Type]
    oxygen: Id[fluid.Type]
    co2: Id[fluid.Type]
    water: Id[fluid.Type]

    def write(writer: Writer, glossary: Glossary) -> Self:
        def define_word(base, **locales):
            return TemplateDisplayText(id=glossary.add(base, **locales))

        return Fluids(
            nitrogen=fluid.Type.gas_like(define_word("Nitrogen"), 28.02).write(writer),
            oxygen=fluid.Type.gas_like(define_word("Oxygen"), 31.99).write(writer),
            co2=fluid.Type.gas_like(define_word("CO2"), 44.01).write(writer),
            water=fluid.Type.aqueous(define_word("Water"), 18.02).write(writer),
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

    def define_common_word(self, ident: object, base, **locales) -> TemplateDisplayText:
        return TemplateDisplayText(id=self.glossary.add_common(ident, base, **locales))

    def define_word(self, base, **locales) -> TemplateDisplayText:
        return TemplateDisplayText(id=self.glossary.add(base, **locales))


def core(ctx: Context, position: Position, rotation: Rotation = Rotation.identity()):
    return Building(
        position=position,
        rotation=rotation,
        scale=Scale.splat(2.0),
        appearance=Appearance(
            label=ctx.define_word("Core"),
            layers=Layers(
                distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
                proximal=PbrLayer(
                    mesh=sphere.Mesh(depth=5), material=common_materials.Glass()
                ),
                interior=PbrLayer(
                    mesh=sphere.Mesh(), material=common_materials.Glass()
                ),
            ),
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
        appearance=Appearance(
            label=ctx.define_word("Garden"),
            layers=Layers(
                distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
                proximal=PbrLayer(
                    mesh=sphere.Mesh(), material=common_materials.Glass()
                ),
                interior=PbrLayer(
                    mesh=sphere.Mesh(), material=common_materials.Glass()
                ),
            ),
        ),
        ambient_facility=Facility(
            fluid_containers=[
                ctx.fluids.ambient_container(max_volume=1000.0, max_pressure=100.0),
            ]
        ),
        other_facilities=[
            Facility(
                inner_scale=Scale(x=0.3, y=0.3, z=0.7),
                appearance=Appearance(
                    label=ctx.define_word("Bushes"),
                    layers=Layers(
                        distal=PbrLayer(
                            mesh=cylinder.Mesh(sides=12),
                            material=common_materials.RoughMonotone(
                                r=0.39, g=0.85, b=0.34
                            ),
                        ),
                        proximal=PbrLayer(
                            mesh=cylinder.Mesh(),
                            material=common_materials.RoughMonotone(
                                r=0.39, g=0.85, b=0.34
                            ),
                        ),
                    ),
                ),
                fluid_containers=[
                    ctx.fluids.ambient_container(max_volume=100.0, max_pressure=100.0),
                ],
            ),
        ],
    )


corridor_name = object()


def corridor(
    ctx: Context,
    alpha: Id[Building],
    beta: Id[Building],
    name: Optional[DisplayText] = None,
    other_ducts: list[Duct] = [],
):
    if name is None:
        name = ctx.define_common_word(corridor_name, "Corridor")

    ambient_volume = 100  # TODO calculate this from corridor length

    return Corridor(
        alpha=alpha,
        beta=beta,
        radius=0.2,
        appearance=Appearance(
            label=name,
            layers=Layers(
                distal=PbrLayer(
                    mesh=cylinder.Mesh(sides=12), material=common_materials.Glass()
                ),
                proximal=PbrLayer(
                    mesh=cylinder.Mesh(), material=common_materials.Glass()
                ),
                interior=PbrLayer(
                    mesh=cylinder.Mesh(), material=common_materials.Glass()
                ),
            ),
        ),
        ambient_duct=Duct(
            fluid_container=ctx.fluids.ambient_container(
                max_volume=ambient_volume, max_pressure=100.0
            ),
        ),
        other_ducts=other_ducts,
    )


fluid_duct_name = object()


def fluid_duct(
    ctx: Context,
    index: int,
    rgb: tuple[float, float, float],
    inner: tuple[float, float],
    radius: float,
) -> Duct:
    return Duct(
        appearance=Appearance(
            label=ConcatDisplayText(
                ctx.define_common_word(fluid_duct_name, "Fluid duct #"),
                LiteralDisplayText(str(index)),
            ),
            layers=Layers(
                distal=PbrLayer(
                    mesh=cylinder.Mesh(sides=12),
                    material=common_materials.RoughMonotone(*rgb),
                ),
                proximal=PbrLayer(
                    mesh=cylinder.Mesh(),
                    material=common_materials.RoughMonotone(*rgb),
                ),
            ),
        ),
        inner_x=inner[0],
        inner_y=inner[1],
        radius=radius,
        # TODO fluid container
    )
