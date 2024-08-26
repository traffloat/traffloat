#!/usr/bin/env python3

import all
import save
import models
import os
import shutil
from os import path


def main():
    proj_root = path.dirname(path.dirname(__file__))
    assets = path.join(proj_root, "assets")
    if path.exists(assets):
        shutil.rmtree(assets)

    os.mkdir(assets)
    os.mkdir(path.join(assets, "shaders"))

    for name, fn in all.scenarios.items():
        print(f"create scenario file {name}.tfsave")
        with save.WriterCtx(assets, name) as writer:
            fn(writer)

    for name, mesh in models.all.items():
        print(f"create asset file {name}: {mesh.hash}")
        with open(path.join(assets, f"{mesh.hash}.glb"), "wb") as f:
            f.write(mesh.buf)

    for file in os.scandir(path.join(proj_root, "shaders")):
        shutil.copyfile(file.path, path.join(assets, "shaders", file.name))


if __name__ == "__main__":
    main()
