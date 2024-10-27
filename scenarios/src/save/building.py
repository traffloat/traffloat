from dataclasses import dataclass, field, KW_ONLY
from typing import Optional, Self

from . import Def, Id, Writer
from .facility import Facility
from .types import Appearance, Position, Rotation, Scale


@dataclass
class Building(Def):
    _: KW_ONLY

    position: Position
    rotation: Rotation = field(default_factory=Rotation.identity)
    scale: Scale = field(default_factory=Scale)

    appearance: Appearance

    ambient_facility: Facility
    other_facilities: list[Facility] = field(default_factory=list)

    id: Optional[Id[Self]] = None

    @staticmethod
    def save_id() -> str:
        return "traffloat.save.Building"

    def write(self, writer: Writer) -> Id[Self]:
        self.id = writer.write(
            Building,
            {
                "transform": {
                    "position": self.position.as_dict(),
                    "rotation": self.rotation.as_dict(),
                    "scale": self.scale.as_dict(),
                },
                "appearance": self.appearance.as_dict(writer),
            },
        )

        self.ambient_facility.write(writer=writer, parent=self.id, is_ambient=True)

        for facility in self.other_facilities:
            facility.write(writer=writer, parent=self.id, is_ambient=False)

        return self.id
