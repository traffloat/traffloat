#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy
import os
import typing

WIDTH = 256
HEIGHT = 256

TEXTURE_SIZE = 16

agg = numpy.zeros((WIDTH, HEIGHT, 3))

class IndexAlloc:
    def __init__(self):
        self.X = 0
        self.max_X = WIDTH / TEXTURE_SIZE
        self.Y = 0
        self.max_Y = HEIGHT / TEXTURE_SIZE

    def alloc(self) -> numpy.ndarray:
        x = self.X * TEXTURE_SIZE
        y = self.Y * TEXTURE_SIZE
        self.X += 1
        if self.X == self.max_X:
            self.X = 0
            self.Y += 1

        return x, y, agg[x:(x + TEXTURE_SIZE), y:(y + TEXTURE_SIZE), :]

index = {}

alloc = IndexAlloc()

os.chdir(os.path.dirname(os.path.realpath(__file__)))

def add_dir(path: str):
    subindex: typing.Dict[str, [
        str,
        typing.Tuple[int, int],
    ]] = {"shape": "cube"}

    last = None

    for direction in ["x", "y", "z"]:
        for side in ["p", "n"]:
            im = plt.imread(os.path.join(path, direction + side + ".png"))
            if last is None or not numpy.all(im == last):
                x, y, region = alloc.alloc()
                region[:] = im
                last = im
            subindex[direction + side] = {
                "x": x,
                "y": y,
                "width": TEXTURE_SIZE,
                "height": TEXTURE_SIZE,
            }
    index[path] = subindex

for path in os.listdir("."):
    if os.path.isfile(os.path.join(path, "xp.png")):
        add_dir(path)

plt.imsave("../static/textures.png", agg.transpose((1, 0, 2)))
with open("../static/textures.png.json", "w") as fh:
    fh.write(json.dumps(index, separators=(",", ":"), indent=1))
