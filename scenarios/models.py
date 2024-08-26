from io import BytesIO
import hashlib
from typing import Callable, Optional
import gltflib
from gltflib import GLTF, GLTFModel, GLBResource
import numpy

all = {}


class Mesh:
    def __init__(self, buf):
        self.buf = buf
        self.hash = hashlib.sha1(buf, usedforsecurity=False).digest().hex()


def register_gltf_mesh(fn: Callable[[], tuple[numpy.ndarray, numpy.ndarray]]):
    return register(lambda: gltf_mesh_fn(fn), name=f"{fn.__module__}:{fn.__name__}")


def gltf_mesh_fn(fn: Callable[[], tuple[numpy.ndarray, numpy.ndarray]]):
    vertices, normals, faces = fn()

    vert_bin = vertices.astype(numpy.float32).tobytes()
    norm_bin = normals.astype(numpy.float32).tobytes()
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
                        ),
                        indices=2,
                    ),
                ],
            )
        ],
        buffers=[
            gltflib.Buffer(byteLength=len(vert_bin) + len(norm_bin) + len(face_bin))
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
                count=faces.shape[0] * 3,
                componentType=gltflib.ComponentType.UNSIGNED_SHORT.value,
                type=gltflib.AccessorType.SCALAR.value,
                min=int(faces.min()),
                max=int(faces.max()),
            ),
        ],
    )
    return GLTF(model=model, resources=[GLBResource(vert_bin + norm_bin + face_bin)])


def register(make: Callable[[], GLTF], name: Optional[str] = None):
    if name is None:
        name = f"{make.__module__}:{make.__name__}"

    if name not in all:
        gltf = make()
        buf = BytesIO()
        gltf.write_glb(buf)

        all[name] = Mesh(buf.getvalue())

    return {"sha": all[name].hash, "index": 0}
