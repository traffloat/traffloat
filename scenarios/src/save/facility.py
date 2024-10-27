from dataclasses import dataclass, field, KW_ONLY
from typing import Optional, Self, TYPE_CHECKING

from . import Def, Id, Writer
from .fluid.container import Container as FluidContainer
from .types import Appearance, Position, Rotation, Scale

if TYPE_CHECKING:
    from .building import Building


@dataclass
class Facility(Def):
    _: KW_ONLY

    inner_position: Position = field(default_factory=Position)
    inner_rotation: Rotation = field(default_factory=Rotation.identity)
    inner_scale: Scale = field(default_factory=Scale)

    appearance: Appearance = field(default_factory=Appearance)

    fluid_containers: list[FluidContainer] = field(default_factory=list)

    id: Optional[Id[Self]] = None

    @staticmethod
    def save_id() -> str:
        return "traffloat.save.Facility"

    def write(
        self, writer: Writer, parent: Id["Building"], is_ambient: bool
    ) -> Id[Self]:
        self.id = writer.write(
            Facility,
            {
                "parent": parent.id,
                "is_ambient": is_ambient,
                "inner": {
                    "position": self.inner_position.as_dict(),
                    "rotation": self.inner_rotation.as_dict(),
                    "scale": self.inner_scale.as_dict(),
                },
                "appearance": self.appearance.as_dict(writer),
            },
        )

        for container in self.fluid_containers:
            container.write(writer, "Facility", self.id)

        return self.id
