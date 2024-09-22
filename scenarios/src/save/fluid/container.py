from dataclasses import dataclass, KW_ONLY
from typing import Self

from .. import Def, Id, Writer
from . import Type


@dataclass
class Container(Def):
    _: KW_ONLY

    max_volume: float
    max_pressure: float

    element_masses: dict[Id[Type], float]

    def save_id() -> str:
        return "traffloat.save.fluid.Container"

    def write(self, writer: Writer, parent_type: str, parent_id: Id[Def]) -> Id[Self]:
        self.id = writer.write(
            Container,
            {
                "owner": {
                    "type": parent_type,
                    "id": parent_id.id,
                },
                "max_volume": self.max_volume,
                "max_pressure": self.max_pressure,
            },
        )

        for ty, mass in self.element_masses.items():
            writer.write(
                ContainerElement,
                {
                    "parent": self.id.id,
                    "ty": ty,
                    "mass": mass,
                },
            )

        return self.id


class ContainerElement(Def):
    def save_id() -> str:
        return "traffloat.save.fluid.ContainerElement"
