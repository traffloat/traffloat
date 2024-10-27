from dataclasses import dataclass, field, KW_ONLY
from typing import Optional, Self, TYPE_CHECKING

from . import Def, Id, Writer
from .fluid.container import Container as FluidContainer
from .types import Appearance

if TYPE_CHECKING:
    from .building import Building


@dataclass
class Duct(Def):
    _: KW_ONLY

    appearance: Appearance = field(default_factory=Appearance)

    radius: float = 1.0
    inner_x: float = 0.0
    inner_y: float = 0.0

    fluid_container: Optional[FluidContainer] = None

    id: Optional[Id[Self]] = None

    @staticmethod
    def save_id() -> str:
        return "traffloat.save.Duct"

    def write(
        self, writer: Writer, parent: Id["Corridor"], is_ambient: bool
    ) -> Id[Self]:
        self.id = writer.write(
            Duct,
            {
                "parent": parent.id,
                "appearance": self.appearance.as_dict(writer),
                "ambient": {
                    "shape": "Ambient",
                }
                if is_ambient
                else {
                    "shape": "Cylindrical",
                    "radius": self.radius,
                    "inner": {"x": self.inner_x, "y": self.inner_y},
                },
            },
        )

        if self.fluid_container is not None:
            self.fluid_container.write(writer, "Duct", self.id)

        return self.id
