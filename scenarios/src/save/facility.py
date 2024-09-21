from dataclasses import dataclass, field, KW_ONLY
from typing import Optional, Self, TYPE_CHECKING

from . import Def, Id, Writer
from .types import CustomDisplayText, DisplayText, Layers, Position, Rotation, Scale

if TYPE_CHECKING:
    from .building import Building


@dataclass
class Facility(Def):
    _: KW_ONLY

    inner_position: Position = field(default_factory=Position)
    inner_rotation: Rotation = field(default_factory=Rotation.identity)
    inner_scale: Scale = field(default_factory=Scale)

    label: DisplayText = field(default_factory=CustomDisplayText)
    layers: Layers = field(default_factory=Layers)

    id: Optional[Id[Self]] = None

    def save_id() -> str:
        return "traffloat.save.Facility"

    def write(self, writer: Writer, parent: Id["Building"], is_ambient: bool):
        writer.write(
            Facility,
            {
                "parent": parent.id,
                "is_ambient": is_ambient,
                "inner": {
                    "position": self.inner_position.as_dict(),
                    "rotation": self.inner_rotation.as_dict(),
                    "scale": self.inner_scale.as_dict(),
                },
                "appearance": {
                    "label": self.label.as_dict(),
                    "distal": self.layers.distal.as_dict(writer),
                    "proximal": self.layers.proximal.as_dict(writer),
                    "interior": self.layers.interior.as_dict(writer),
                },
            },
        )
