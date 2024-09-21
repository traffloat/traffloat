from abc import abstractmethod
from dataclasses import dataclass, field
from typing import Self

from ..assets import Material, Mesh
from . import Writer


@dataclass
class Position:
    x: float = 0.0
    y: float = 0.0
    z: float = 0.0

    def as_dict(self):
        return {"x": self.x, "y": self.y, "z": self.z}


@dataclass
class Rotation:
    x: float
    y: float
    z: float
    w: float

    def identity() -> Self:
        return Rotation(0.0, 0.0, 0.0, 1.0)

    def as_dict(self):
        return {"xyzw": [self.x, self.y, self.z, self.w]}


@dataclass
class Scale:
    x: float = 1.0
    y: float = 1.0
    z: float = 1.0

    def splat(v: float) -> Self:
        return Scale(v, v, v)

    def as_dict(self):
        return {"x": self.x, "y": self.y, "z": self.z}


class DisplayText:
    @abstractmethod
    def as_dict(self):
        pass


@dataclass
class CustomDisplayText(DisplayText):
    text: str = ""

    def as_dict(self):
        return {"type": "Custom", "value": self.text}


class Layer:
    @abstractmethod
    def as_dict(self, writer: Writer):
        """
        Converts the layer to a dictionary.
        """


class NullLayer:
    def as_dict(self, writer: Writer):
        return {"type": "Null"}


@dataclass
class PbrLayer:
    mesh: Mesh
    material: Material

    def as_dict(self, writer: Writer):
        return {
            "type": "Pbr",
            "mesh": self.mesh.use(writer.pool),
            "material": self.material.use(writer.pool),
        }


@dataclass
class Layers:
    distal: Layer = field(default_factory=NullLayer)
    proximal: Layer = field(default_factory=NullLayer)
    interior: Layer = field(default_factory=NullLayer)

    def as_dict(self, writer: Writer):
        return {
            "distal": self.distal.as_dict(writer),
            "proximal": self.proximal.as_dict(writer),
            "interior": self.interior.as_dict(writer),
        }
