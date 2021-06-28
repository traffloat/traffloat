#!/usr/bin/env python3

import json
import matplotlib.pyplot as plt
import numpy
import os
import typing

agg = numpy.zeros((256, 256, 3))

next_cell = 0
index = {}

LN_TEXTURE_SIZE = 4
TEXTURE_SIZE = 1 << 4

def add_dir(path: str):
    global next_cell

    subindex: typing.Dict[str, [
        str,
        typing.Tuple[int, int],
    ]] = {"shape": "cube"}

    last = None

    for direction in ["x", "y", "z"]:
        for side in ["p", "n"]:
            im = plt.imread(os.path.join(path, direction + side + ".png"))
            if last is None or not numpy.all(im == last):
                x = next_cell >> LN_TEXTURE_SIZE
                y = next_cell & (TEXTURE_SIZE - 1)
                x <<= LN_TEXTURE_SIZE
                y <<= LN_TEXTURE_SIZE
                next_cell += 1
                agg[x:(x + TEXTURE_SIZE), y:(y + TEXTURE_SIZE), :] = im
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

plt.imsave("../static/textures.png", agg)
with open("../static/textures.png.json", "w") as fh:
    fh.write(json.dumps(index, separators=(",", ":"), indent=1))
