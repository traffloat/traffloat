from dataclasses import dataclass

import gltflib
import numpy
from gltflib import GLTF, GLTFModel

from . import assets


@dataclass
class Mesh(assets.Mesh):
    """
    A cylinder where both ends are n-gons at z=1 and z=-1 respectively, with radius 1.
    Also produces a UV map expecting an image of the following structure:
    (TOP) (BOTTOM) [SIDE]
    where (TOP) and (BOTTOM) are radius-1 circles and [SIDE] is a 2pi x 2 rectangle.
    """

    sides: int = 16

    def id(self):
        return f"cylinder(sides={self.sides})"

    def generate(self):
        top_corner_angles = numpy.arange(
            start=0.0, stop=numpy.pi * 2.0, step=numpy.pi * 2.0 / float(self.sides)
        )
        top_corners = numpy.zeros((self.sides, 3))
        top_corners[:, 0] = numpy.cos(top_corner_angles)
        top_corners[:, 1] = numpy.sin(top_corner_angles)
        top_corners[:, 2] = 1.0

        top_verts = numpy.zeros((self.sides + 1, 3))
        top_verts[: self.sides, :] = top_corners
        top_verts[self.sides, :] = numpy.array([0.0, 0.0, 1.0])

        top_normals = numpy.zeros((self.sides + 1, 3))
        top_normals[:, 2] = 1.0

        top_uvs = top_verts[:, (0, 1)].copy()

        top_circle_faces = numpy.hstack(
            [
                numpy.arange(self.sides, dtype=int).reshape((self.sides, 1)),
                (numpy.arange(self.sides, dtype=int).reshape((self.sides, 1)) + 1)
                % self.sides,
                numpy.repeat(self.sides, self.sides).reshape((self.sides, 1)),
            ]
        )

        bottom_corner_angles = numpy.arange(
            start=0.0, stop=numpy.pi * 2.0, step=numpy.pi * 2.0 / float(self.sides)
        ) + numpy.pi / float(self.sides)
        bottom_corners = numpy.zeros((self.sides, 3))
        bottom_corners[:, 0] = numpy.cos(bottom_corner_angles)
        bottom_corners[:, 1] = numpy.sin(bottom_corner_angles)
        bottom_corners[:, 2] = -1.0

        bottom_verts = numpy.zeros((self.sides + 1, 3))
        bottom_verts[: self.sides, :] = bottom_corners
        bottom_verts[self.sides, :] = numpy.array([0.0, 0.0, -1.0])

        bottom_normals = numpy.zeros((self.sides + 1, 3))
        bottom_normals[:, 2] = -1.0

        bottom_uvs = bottom_verts[:, (0, 1)].copy()
        bottom_uvs[:, 0] += 1.0

        bottom_circle_faces = numpy.hstack(
            [
                (numpy.arange(self.sides, dtype=int).reshape((self.sides, 1)) + 1)
                % self.sides,
                numpy.arange(self.sides, dtype=int).reshape((self.sides, 1)),
                numpy.repeat(self.sides, self.sides).reshape((self.sides, 1)),
            ]
        )

        side_verts = numpy.vstack([top_corners, bottom_corners])

        side_normals = side_verts.copy()
        side_normals[:, 2] = 0.0

        side_uvs = numpy.zeros((self.sides * 2, 2))
        side_uvs[: self.sides, 0] = top_corner_angles + 2.0
        side_uvs[: self.sides, 1] = 0.0
        side_uvs[self.sides :, 0] = bottom_corner_angles + 2.0
        side_uvs[self.sides :, 1] = 1.0

        side_faces = numpy.zeros((self.sides * 2, 3))

        side_faces[: self.sides, 0] = numpy.arange(0, self.sides, dtype=int)
        side_faces[: self.sides, 1] = numpy.arange(
            self.sides, self.sides * 2, dtype=int
        )
        side_faces[: self.sides, 2] = (side_faces[: self.sides, 0] + 1) % self.sides

        side_faces[self.sides :, 0] = side_faces[: self.sides, 2]
        side_faces[self.sides :, 1] = side_faces[: self.sides, 1]
        side_faces[self.sides :, 2] = side_faces[: self.sides, 1] + 1
        side_faces[self.sides * 2 - 1, 2] = side_faces[0, 1]

        verts = numpy.vstack([top_verts, bottom_verts, side_verts])
        normals = numpy.vstack([top_normals, bottom_normals, side_normals])

        uvs = numpy.vstack([top_uvs, bottom_uvs, side_uvs])
        uvs[:, 0] /= numpy.pi * 2 + 2.0

        faces = numpy.vstack(
            [
                top_circle_faces,
                bottom_circle_faces + top_verts.shape[0],
                side_faces + top_verts.shape[0] + bottom_verts.shape[0],
            ]
        )
        return self.generate_with(
            vertices=verts,
            normals=normals,
            uvs=uvs,
            faces=faces,
        )
