import models
import numpy
import gltflib
from gltflib import GLTF, GLTFModel


def mesh():
    return mesh_with_depth(4)


def mesh_with_depth(depth):
    # Start with a tetrahedron with spherical coordinates
    verts = numpy.array(
        [
            [0.0, numpy.pi / 2.0],
            [0.0, numpy.pi / 2.0 - numpy.acos(-1 / 3)],
            [numpy.pi * 2.0 / 3.0, numpy.pi / 2.0 - numpy.acos(-1 / 3)],
            [numpy.pi * 4.0 / 3.0, numpy.pi / 2.0 - numpy.acos(-1 / 3)],
        ]
    )
    faces = numpy.array(
        [
            [0, 1, 2],
            [0, 2, 3],
            [0, 3, 1],
            [1, 3, 2],
        ]
    )

    for _ in range(depth):
        verts, faces = recurse(verts, faces)

    theta = verts[:, 0].reshape([-1, 1])
    phi = verts[:, 1].reshape([-1, 1])

    verts = numpy.hstack(
        [
            numpy.cos(theta) * numpy.cos(phi),
            numpy.sin(theta) * numpy.cos(phi),
            numpy.sin(phi),
        ]
    )
    return verts, verts, faces


def recurse(verts, faces):
    midpt_index = {}

    new_verts = numpy.zeros(
        (verts.shape[0] + faces.shape[0] * 3 // 2, 2)
    )  # each face has 3 edges, each edge is shared by 2 faces
    new_verts[: verts.shape[0], :] = verts
    next_new_vert = verts.shape[0]

    new_faces = numpy.zeros((faces.shape[0] * 4, 3), dtype=int)

    for i, face in enumerate(faces):
        for pt0, pt1 in [(0, 1), (1, 2), (2, 0)]:
            midpt_key = (face[pt0], face[pt1])
            if midpt_key in midpt_index:
                vert_index = midpt_index[midpt_key]
            else:
                vert_index = next_new_vert
                next_new_vert += 1
                midpt_index[midpt_key] = vert_index
                midpt_index[(face[pt1], face[pt0])] = vert_index

                new_verts[vert_index] = sph_midpt(
                    verts[midpt_key[0], :], verts[midpt_key[1], :]
                )

        new_faces[i * 4, :] = (
            midpt_index[(face[0], face[1])],
            midpt_index[(face[1], face[2])],
            midpt_index[(face[2], face[0])],
        )
        new_faces[i * 4 + 1, :] = (
            face[0],
            midpt_index[(face[0], face[1])],
            midpt_index[(face[2], face[0])],
        )
        new_faces[i * 4 + 2, :] = (
            face[1],
            midpt_index[(face[1], face[2])],
            midpt_index[(face[0], face[1])],
        )
        new_faces[i * 4 + 3, :] = (
            face[2],
            midpt_index[(face[0], face[2])],
            midpt_index[(face[2], face[1])],
        )

    return new_verts, new_faces


def material_glass():
    return GLTF(model=GLTFModel(
        asset=gltflib.Asset(version="2.0"),
        materials=[
            gltflib.Material(
                name="Material0",
                pbrMetallicRoughness={
                    "baseColorFactor": [1., 1., 1., 0.5],
                    "metallicFactor": 0.,
                    "roughnessFactor": 0.07,
                },
            ),
        ],
    ), resources=[])


def sph_midpt(a, b):
    return car_to_sph((sph_to_car(a) + sph_to_car(b)) / 2)


def sph_to_car(v):
    return numpy.array(
        [
            numpy.cos(v[0]) * numpy.cos(v[1]),
            numpy.sin(v[0]) * numpy.cos(v[1]),
            numpy.sin(v[1]),
        ]
    )


def car_to_sph(v):
    return numpy.array(
        [
            numpy.atan2(v[1], v[0]),
            numpy.atan2(v[2], numpy.sqrt(v[0] ** 2 + v[1] ** 2)),
        ]
    )


def glass_material():
    pass
