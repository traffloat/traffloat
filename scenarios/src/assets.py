import hashlib
from abc import abstractmethod
from dataclasses import dataclass, field
from io import BytesIO
from typing import Callable, Optional

import gltflib
import numpy
from gltflib import GLBResource, GLTF, GLTFModel


@dataclass
class Serialized:
    id: str
    buf: bytes
    hash: str


class Asset:
    @abstractmethod
    def id(self) -> str:
        """
        Uniquely identifies the asset for deduplication.
        """

    @abstractmethod
    def generate(self) -> GLTF:
        """
        Generates the asset as a GLTF object.
        This method is called by use() lazily.
        """


@dataclass
class Pool:
    """
    A pool of reusable assets.
    """

    all: dict[str, Serialized] = field(default_factory=dict)

    def register(self, asset: Asset):
        id = asset.id()
        if id in self.all:
            return self.all[id].hash

        gltf = asset.generate()
        buf = BytesIO()
        gltf.write_glb(buf)

        hash = hashlib.sha1(buf.getvalue(), usedforsecurity=False).digest().hex()

        self.all[id] = Serialized(id=id, buf=buf.getvalue(), hash=hash)
        return hash


class Mesh(Asset):
    def use(self, pool: Pool) -> dict:
        hash = pool.register(self)
        return {"sha": hash, "mesh": 0, "primitive": 0}

    def generate_with(
        self,
        vertices: numpy.ndarray,
        normals: numpy.ndarray,
        uvs: numpy.ndarray,
        faces: numpy.ndarray,
    ):
        vert_bin = vertices.astype(numpy.float32).tobytes()
        norm_bin = normals.astype(numpy.float32).tobytes()
        uv_bin = uvs.astype(numpy.float32).tobytes()
        face_bin = faces.astype(numpy.uint16).tobytes()

        model = GLTFModel(
            asset=gltflib.Asset(version="2.0"),
            scenes=[gltflib.Scene(nodes=[0])],
            nodes=[gltflib.Node(mesh=0)],
            meshes=[
                gltflib.Mesh(
                    name="Mesh0",
                    primitives=[
                        gltflib.Primitive(
                            attributes=gltflib.Attributes(
                                POSITION=0,
                                NORMAL=1,
                                TEXCOORD_0=2,
                            ),
                            indices=3,
                        ),
                    ],
                )
            ],
            buffers=[
                gltflib.Buffer(
                    byteLength=len(vert_bin)
                    + len(norm_bin)
                    + len(uv_bin)
                    + len(face_bin)
                ),
            ],
            bufferViews=[
                gltflib.BufferView(
                    buffer=0,
                    byteOffset=0,
                    byteLength=len(vert_bin),
                    target=gltflib.BufferTarget.ARRAY_BUFFER.value,
                ),
                gltflib.BufferView(
                    buffer=0,
                    byteOffset=len(vert_bin),
                    byteLength=len(norm_bin),
                    target=gltflib.BufferTarget.ARRAY_BUFFER.value,
                ),
                gltflib.BufferView(
                    buffer=0,
                    byteOffset=len(vert_bin) + len(norm_bin),
                    byteLength=len(uv_bin),
                    target=gltflib.BufferTarget.ARRAY_BUFFER.value,
                ),
                gltflib.BufferView(
                    buffer=0,
                    byteOffset=len(vert_bin) + len(norm_bin) + len(uv_bin),
                    byteLength=len(face_bin),
                    target=gltflib.BufferTarget.ELEMENT_ARRAY_BUFFER.value,
                ),
            ],
            accessors=[
                gltflib.Accessor(
                    bufferView=0,
                    count=vertices.shape[0],
                    componentType=gltflib.ComponentType.FLOAT.value,
                    type=gltflib.AccessorType.VEC3.value,
                    min=vertices.min(axis=0).tolist(),
                    max=vertices.max(axis=0).tolist(),
                ),
                gltflib.Accessor(
                    bufferView=1,
                    count=normals.shape[0],
                    componentType=gltflib.ComponentType.FLOAT.value,
                    type=gltflib.AccessorType.VEC3.value,
                    min=normals.min(axis=0).tolist(),
                    max=normals.max(axis=0).tolist(),
                ),
                gltflib.Accessor(
                    bufferView=2,
                    count=uvs.shape[0],
                    componentType=gltflib.ComponentType.FLOAT.value,
                    type=gltflib.AccessorType.VEC2.value,
                    min=uvs.min(axis=0).tolist(),
                    max=uvs.max(axis=0).tolist(),
                ),
                gltflib.Accessor(
                    bufferView=3,
                    count=faces.shape[0] * 3,
                    componentType=gltflib.ComponentType.UNSIGNED_SHORT.value,
                    type=gltflib.AccessorType.SCALAR.value,
                    min=int(faces.min()),
                    max=int(faces.max()),
                ),
            ],
        )
        return GLTF(
            model=model,
            resources=[GLBResource(vert_bin + norm_bin + uv_bin + face_bin)],
        )


class Material(Asset):
    def use(self, pool: Pool) -> dict:
        hash = pool.register(self)
        return {"sha": hash, "index": 0}
