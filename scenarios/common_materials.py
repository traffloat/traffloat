from dataclasses import dataclass
import assets
import gltflib
from gltflib import GLTFModel, GLTF


@dataclass
class Glass(assets.Material):
    def id(self):
        return "glass"

    def generate(self):
        return GLTF(
            model=GLTFModel(
                asset=gltflib.Asset(version="2.0"),
                materials=[
                    gltflib.Material(
                        name="Material0",
                        alphaMode=gltflib.AlphaMode.MASK.name,
                        alphaCutoff=0.1,
                        doubleSided=True,
                        pbrMetallicRoughness={
                            "baseColorFactor": [1.0, 1.0, 1.0, 0.2],
                            "metallicFactor": 0.0,
                            "roughnessFactor": 0.05,
                        },
                        extensions={
                            "KHR_materials_transmission": {
                                "transmissionFactor": 0.9,
                            },
                        },
                    ),
                ],
            ),
            resources=[],
        )


@dataclass
class RoughMonotone(assets.Material):
    r: int
    g: int
    b: int

    def id(self):
        return f"rough_monotone(rgb({self.r}, {self.g}, {self.b}))"

    def generate(self):
        return GLTF(
            model=GLTFModel(
                asset=gltflib.Asset(version="2.0"),
                materials=[
                    gltflib.Material(
                        name="Material0",
                        pbrMetallicRoughness={
                            "baseColorFactor": [self.r, self.g, self.b, 1.0],
                            "metallicFactor": 1.0,
                        },
                    ),
                ],
            ),
            resources=[],
        )
