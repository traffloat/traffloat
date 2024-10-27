from dataclasses import dataclass, field, KW_ONLY
from typing import Optional, Self

from . import Def, Id, Writer
from .building import Building
from .duct import Duct
from .types import Appearance


@dataclass
class Corridor(Def):
    _: KW_ONLY

    alpha: Id[Building]
    beta: Id[Building]

    radius: float
    appearance: Appearance

    ambient_duct: Duct
    other_ducts: list[Duct] = field(default_factory=list)

    id: Optional[Id[Self]] = None

    @staticmethod
    def save_id() -> str:
        return "traffloat.save.Corridor"

    def write(self, writer: Writer) -> Id[Self]:
        self.id = writer.write(
            Corridor,
            {
                "endpoints": {
                    "alpha": self.alpha.id,
                    "beta": self.beta.id,
                },
                "radius": self.radius,
                "appearance": self.appearance.as_dict(writer),
            },
        )

        # self.ambient_duct.write(writer=writer, parent=self.id, is_ambient=True)

        for duct in self.other_ducts:
            duct.write(writer=writer, parent=self.id, is_ambient=False)

        return self.id
