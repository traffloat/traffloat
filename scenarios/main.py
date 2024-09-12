#!/usr/bin/env python3

import all
import save
import assets
import os
import shutil
from os import path


def main():
    proj_root = path.dirname(path.dirname(__file__))
    assets_dir = path.join(proj_root, "assets")
    if path.exists(assets_dir):
        shutil.rmtree(assets_dir)

    os.mkdir(assets_dir)
    os.mkdir(path.join(assets_dir, "shaders"))

    pool = assets.Pool()

    for name, fn in all.scenarios.items():
        print(f"create scenario file {name}.tfsave")
        with save.WriterCtx(assets_dir, name) as writer:
            fn(writer, pool)

    for name, mesh in pool.all.items():
        print(f"create asset file {name}: {mesh.hash}")
        with open(path.join(assets_dir, f"{mesh.hash}.glb"), "wb") as f:
            f.write(mesh.buf)

    for file in os.scandir(path.join(proj_root, "shaders")):
        shutil.copyfile(file.path, path.join(assets_dir, "shaders", file.name))


if __name__ == "__main__":
    main()
