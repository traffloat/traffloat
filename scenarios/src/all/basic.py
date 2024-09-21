from .. import assets, common_materials, cylinder, sphere
from ..save import Writer
from ..save.building import Building
from ..save.facility import Facility
from ..save.types import (
    CustomDisplayText,
    Layer,
    Layers,
    PbrLayer,
    Position,
    Rotation,
    Scale,
)


def write_scenario(writer: Writer):
    core(position=Position(x=-2.0, y=0.0, z=5.0)).write(writer)
    garden(position=Position(x=2.0, y=0.0, z=5.0)).write(writer)


def core(position: Position, rotation: Rotation = Rotation.identity()):
    return Building(
        position=position,
        rotation=rotation,
        scale=Scale.splat(2.0),
        label=CustomDisplayText("Core"),
        layers=Layers(
            distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            proximal=PbrLayer(
                mesh=sphere.Mesh(depth=5), material=common_materials.Glass()
            ),
            interior=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
        ),
        ambient_facility=Facility(),
    )


def garden(position: Position, rotation: Rotation = Rotation.identity()):
    return Building(
        position=position,
        rotation=rotation,
        label=CustomDisplayText("Garden"),
        layers=Layers(
            distal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            proximal=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
            interior=PbrLayer(mesh=sphere.Mesh(), material=common_materials.Glass()),
        ),
        ambient_facility=Facility(),
        other_facilities=[
            Facility(
                inner_scale=Scale(x=0.3, y=0.3, z=0.7),
                label=CustomDisplayText("Bushes"),
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
